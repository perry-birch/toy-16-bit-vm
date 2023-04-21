use core::{fmt, fmt::Write, mem};

use heapless::String;

use crate::{
    Instructions, Instructions::*, MachineError, MemoryWindow, Ptr, Registers, Registers::*,
    VMSize, REGISTER_COUNT,
};

#[derive(Clone)]
pub struct Machine<const MEMORY: usize>
where
    [(); MEMORY * mem::size_of::<u8>()]:,
{
    pub registers: [VMSize; REGISTER_COUNT as usize],
    pub stack_frame_size: VMSize,
    pub memory: [u8; MEMORY * mem::size_of::<u8>()],
}

impl<const MEMORY: usize> Machine<MEMORY>
where
    [(); MEMORY * mem::size_of::<u8>()]:,
{
    fn new() -> Self {
        let mut machine = Machine {
            registers: [0; REGISTER_COUNT as usize],
            stack_frame_size: 0,
            memory: [0; MEMORY * mem::size_of::<u8>()],
        };
        // Initialize the stack and frame pointers to the end of the main memory region for now
        machine.registers[SP as usize] = (MEMORY - 1 - 1) as VMSize;
        machine.registers[FP as usize] = (MEMORY - 1 - 1) as VMSize;
        machine
    }

    #[inline]
    pub fn fetch(&mut self) -> u8 {
        let instruction_address = self.registers[IP as usize];
        let instruction = self.memory[instruction_address as usize];
        self.registers[IP as usize] += 1;
        instruction
    }

    #[inline]
    pub fn fetch16(&mut self) -> u16 {
        let instruction_address = Ptr(self.registers[IP as usize]);
        let result = self.get16(instruction_address);
        self.registers[IP as usize] += 2;
        result
    }

    #[inline]
    pub fn get(&self, addr: Ptr) -> u8 {
        self.memory[addr.0 as usize]
    }

    #[inline]
    pub fn get16(&self, addr: Ptr) -> u16 {
        let high = self.get(addr);
        let low = self.get(addr + 1);
        (high as u16) << 8 | low as u16
    }

    #[inline]
    pub fn set8(&mut self, addr: Ptr, data: u8) {
        self.memory[addr.0 as usize] = data;
    }

    #[inline]
    pub fn set16(&mut self, addr: Ptr, data: u16) {
        self.set8(addr, (data >> 8) as u8);
        self.set8(addr + 1, data as u8);
    }

    #[inline]
    pub fn fetch_register_id(&mut self) -> Result<Registers, MachineError> {
        let reg = self.fetch().try_into()?;
        Ok(reg)
    }

    #[inline]
    pub fn push(&mut self, value: u16) {
        let sp_addr = Ptr(self.registers[SP as usize]);
        self.set16(sp_addr, value);
        self.registers[SP  as usize] -= 2;
        self.stack_frame_size += 2;
    }

    #[inline]
    pub fn pop(&mut self) -> u16 {
        self.registers[SP as usize] += 2;
        let stack_addr = Ptr(self.registers[SP as usize]);
        self.stack_frame_size -= 2;
        self.get16(stack_addr)
    }

    #[inline]
    pub fn push_state(&mut self) {
        // Capture the current register state on the stack
        for reg in R1 as usize..=R8 as usize {
            self.push(self.registers[reg]);
        }
        // Capture the current instruction pointer on the stack
        self.push(self.registers[IP as usize]);
        // Prepare and reset the stack frame values
        self.push(self.stack_frame_size + 2);
        self.registers[FP as usize] = self.registers[SP as usize];
        self.stack_frame_size = 0;
    }

    #[inline]
    pub fn pop_state(&mut self) {
        let frame_pointer_addr = self.registers[FP as usize];
        self.registers[SP as usize] = frame_pointer_addr;
        self.stack_frame_size = self.pop();
        // Restore the prior instruction pointer from the stack
        self.registers[IP as usize] = self.pop();
        // Restore the prior register state from the stack
        for reg in (R1 as usize..=R8 as usize).rev() {
            self.registers[reg] = self.pop();
        }
        // Account for args from the prior function call
        let n_args = self.pop();
        for _arg in 0..n_args {
            self.pop();
        }
        self.registers[FP as usize] = frame_pointer_addr + self.stack_frame_size;
    }

    pub fn get_window(&self, addr: Ptr, len: VMSize) -> MemoryWindow {
        let data = &self.memory[addr.0 as usize..addr.0 as usize + len as usize];
        MemoryWindow { addr, data }
    }

    pub fn execute(&mut self, instruction: Instructions) -> Result<(), MachineError> {
        match instruction {
            MoveLitToReg => {
                let lit_value = self.fetch16();
                let reg_dest = self.fetch_register_id()?;
                self.registers[reg_dest as usize] = lit_value;
            }
            MoveRegToReg => {
                let reg_src = self.fetch_register_id()?;
                let reg_dest = self.fetch_register_id()?;
                let value: VMSize = self.registers[reg_src as usize];
                self.registers[reg_dest as usize] = value;
            }
            MoveRegToMem => {
                let reg_src = self.fetch_register_id()?;
                let addr_dest = Ptr(self.fetch16());
                let value = self.registers[reg_src as usize];
                self.set16(addr_dest, value);
            }
            MoveMemToReg => {
                let addr_src = Ptr(self.fetch16());
                let reg_dest = self.fetch_register_id()?;
                let value = self.get16(addr_src);
                self.registers[reg_dest as usize] = value;
            }
            AddRegReg => {
                let reg_1 = self.fetch_register_id()?;
                let reg_2 = self.fetch_register_id()?;
                let val_1: VMSize = self.registers[reg_1 as usize];
                let val_2: VMSize = self.registers[reg_2 as usize];
                self.registers[ACC as usize] = val_1 + val_2;
            }
            JmpNotEq => {
                let value = self.fetch16();
                let addr = Ptr(self.fetch16());
                if value != self.registers[ACC as usize] {
                    self.registers[IP as usize] = addr.0;
                }
            }
            PushLit => {
                let value = self.fetch16();
                self.push(value);
            }
            PushReg => {
                let reg = self.fetch_register_id()?;
                let value = self.registers[reg as usize];
                self.push(value);
            }
            Pop => {
                let reg = self.fetch_register_id()?;
                self.registers[reg as usize] = self.pop();
            }
            CallLit => {
                let subroutine_addr = self.fetch16();
                self.push_state();
                self.registers[IP as usize] = subroutine_addr;
            }
            CallReg => {
                let reg = self.fetch_register_id()?;
                let subroutine_addr = self.registers[reg as usize];
                self.push_state();
                self.registers[IP as usize] = subroutine_addr;
            }
            Ret => {
                self.pop_state();
            }
            _ => todo!(),
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), MachineError> {
        let instruction = self.fetch().try_into()?;
        self.execute(instruction)
    }
}

impl<const MEMORY: usize> fmt::Debug for Machine<MEMORY>
where
    [(); MEMORY * mem::size_of::<u8>()]:,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = f.debug_struct("Machine");
        for i in 0..REGISTER_COUNT {
            let register =
                Registers::try_from(i).expect("index should not be able to exceed register count");
            let mut register_name: String<3> = String::new();
            write!(register_name, "{register:?}")?;
            let mut register_value: String<6> = String::new();
            write!(
                register_value,
                "{:#06X?}",
                self.registers[register as usize]
            )?;
            result.field(&register_name, &register_value);
        }
        result.field("memory(bytes)", &self.memory.len()).finish()
    }
}

impl Default for Machine<{ crate::DEFAULT_MEMORY_LENGTH }> {
    fn default() -> Self {
        Machine::new()
    }
}
