#![cfg_attr(not(test), no_std)]
#![feature(lint_reasons)]
#![allow(incomplete_features, reason = "known risk")]
#![feature(generic_const_exprs)]

mod machine;
pub use machine::*;
mod memory_window;
pub use memory_window::*;
mod ptr;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
pub use ptr::*;

pub type VMSize = u16;

pub const REGISTER_COUNT: u8 = Registers::R8 as u8 + 1;
pub const DEFAULT_MEMORY_LENGTH: usize = u16::MAX as usize;

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Instructions {
    MoveLitToReg = 0x10,
    MoveRegToReg = 0x11,
    MoveRegToMem = 0x12,
    MoveMemToReg = 0x13,
    AddRegReg = 0x14,
    /// Evaluates a value and modifies the IP (Instruction Pointer) to a
    /// provided address on not equal
    JmpNotEq = 0x15,
    /// Pushes a literal from the instructions onto the stack
    PushLit = 0x17,
    /// Pushes the current value in a specified register onto the stack
    PushReg = 0x18,
    /// Moves the stack points by one value to remove the item at the top
    Pop = 0x19,
    /// Stashes the current machine state on the stack and moves the IP
    /// to the location specified from the next u16 instructions literal
    CallLit = 0x5E,
    /// Stashes the current machine state on the stack and moves the IP
    /// to the location specified from the register identity provided
    CallReg = 0x5F,
    /// Resets the machine state from the last stack fram values and moves
    /// the IP back to the prior instruction location
    Ret = 0x60,
    /// Aborts the machine runtime
    Hlt = 0xFF,
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Registers {
    /// [IP] Instruction Pointer holds a pointer to the current location
    /// the machine should load instructions from
    IP = 0x00,
    /// [SP] Stack Pointer tracks the current position of the stack
    /// within main memory
    SP = 0x01,
    /// [FP] Frame Pointer enables jumping by tracking where in the stack
    /// to unroll prior state back into registers
    FP = 0x02,
    /// [ACC] Accumulator is the standard destination for storing the
    /// result of operations
    ACC = 0x03,
    R1 = 0x04,
    R2 = 0x05,
    R3 = 0x06,
    R4 = 0x07,
    R5 = 0x08,
    R6 = 0x09,
    R7 = 0x0A,
    R8 = 0x0B,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MachineError {
    InvalidInstruction(u8),
    InvalidRegister(u8),
}

impl From<TryFromPrimitiveError<Instructions>> for MachineError {
    fn from(value: TryFromPrimitiveError<Instructions>) -> Self {
        MachineError::InvalidInstruction(value.number)
    }
}

impl From<TryFromPrimitiveError<Registers>> for MachineError {
    fn from(value: TryFromPrimitiveError<Registers>) -> Self {
        MachineError::InvalidRegister(value.number)
    }
}
#[cfg(test)]
mod should {
    use crate::{Instructions::*, Machine, Ptr, Registers::*, VMSize, DEFAULT_MEMORY_LENGTH};

    fn print_machine_state(machine: &Machine<DEFAULT_MEMORY_LENGTH>, windows: &[(String, Ptr, VMSize)]) {
        let instruction_window = machine.get_window(Ptr(0), 48);
        // let heap_window = machine.get_window(Ptr(256), 24);
        let stack_window = machine.get_window(Ptr(DEFAULT_MEMORY_LENGTH as VMSize - 48), 48);
        println!("\n{machine:?}");
        println!("INSTRUCTIONS:\n{instruction_window:#?}");
        // println!("HEAP:\n{heap_window:#?}");
        println!("STACK:\n{stack_window:#?}");

        for window_def in windows {
            let window = machine.get_window(window_def.1, window_def.2);
            println!("WINDOW [{:?}]\n{window:#?}", window_def.0);
        }
    }

    #[allow(dead_code)]
    pub fn counter_program<const MEMORY: usize>(machine: &mut Machine<MEMORY>)
    where
        [(); MEMORY * core::mem::size_of::<u8>()]:
    {
        let mut i = Ptr(0);

        machine.set8(i.inc(), MoveMemToReg.into());
        machine.set8(i.inc(), 0x01);
        machine.set8(i.inc(), 0x00);
        machine.set8(i.inc(), R1.into());

        machine.set8(i.inc(), MoveLitToReg.into());
        machine.set8(i.inc(), 0x00);
        machine.set8(i.inc(), 0x01);
        machine.set8(i.inc(), R2.into());

        machine.set8(i.inc(), AddRegReg.into());
        machine.set8(i.inc(), R1.into());
        machine.set8(i.inc(), R2.into());

        machine.set8(i.inc(), MoveRegToMem.into());
        machine.set8(i.inc(), ACC.into());
        machine.set8(i.inc(), 0x01);
        machine.set8(i.inc(), 0x00);

        machine.set8(i.inc(), JmpNotEq.into());
        machine.set8(i.inc(), 0x00);
        machine.set8(i.inc(), 0x03);
        machine.set8(i.inc(), 0x00);
        machine.set8(i.inc(), 0x00);
    }

    #[allow(dead_code)]
    pub fn swap_registers_program<const MEMORY: usize>(machine: &mut Machine<MEMORY>)
    where
        [(); MEMORY * core::mem::size_of::<u8>()]:
    {
        let mut i = Ptr(0);

        machine.set8(i.inc(), MoveLitToReg.into());
        machine.set8(i.inc(), 0x12);
        machine.set8(i.inc(), 0x34);
        machine.set8(i.inc(), R1.into());

        machine.set8(i.inc(), MoveLitToReg.into());
        machine.set8(i.inc(), 0x56);
        machine.set8(i.inc(), 0x78);
        machine.set8(i.inc(), R2.into());

        machine.set8(i.inc(), PushReg.into());
        machine.set8(i.inc(), R1.into());

        machine.set8(i.inc(), PushReg.into());
        machine.set8(i.inc(), R2.into());

        machine.set8(i.inc(), Pop.into());
        machine.set8(i.inc(), R1.into());

        machine.set8(i.inc(), Pop.into());
        machine.set8(i.inc(), R2.into());
    }

    #[allow(dead_code)]
    pub fn stack_frame_program<const MEMORY: usize>(machine: &mut Machine<MEMORY>)
    where
        [(); MEMORY * core::mem::size_of::<u8>()]:
    {
        let subroutine_addr: u16 = 0x3000;
        let mut i = Ptr(0);

        // Populate the stack with some values

        machine.set8(i.inc(), PushLit.into());
        machine.set8(i.inc(), 0x33);
        machine.set8(i.inc(), 0x33);

        machine.set8(i.inc(), PushLit.into());
        machine.set8(i.inc(), 0x22);
        machine.set8(i.inc(), 0x22);

        machine.set8(i.inc(), PushLit.into());
        machine.set8(i.inc(), 0x11);
        machine.set8(i.inc(), 0x11);

        // Populate some registers with values to check for restore

        machine.set8(i.inc(), MoveLitToReg.into());
        machine.set8(i.inc(), 0x12);
        machine.set8(i.inc(), 0x34);
        machine.set8(i.inc(), R1.into());

        machine.set8(i.inc(), MoveLitToReg.into());
        machine.set8(i.inc(), 0x56);
        machine.set8(i.inc(), 0x78);
        machine.set8(i.inc(), R4.into());

        // Push arg count of zero

        machine.set8(i.inc(), PushLit.into());
        machine.set8(i.inc(), 0x00);
        machine.set8(i.inc(), 0x00);

        machine.set8(i.inc(), CallLit.into());
        machine.set16(i.inc_by(2), subroutine_addr);

        machine.set8(i.inc(), PushLit.into());
        machine.set16(i.inc_by(2), 0x4444);

        machine.set8(i.inc(), PushLit.into());
        machine.set16(i.inc_by(2), 0x5555);

        // Subroutine...
        i = Ptr(subroutine_addr);

        machine.set8(i.inc(), PushLit.into());
        machine.set16(i.inc_by(2), 0x0102);

        machine.set8(i.inc(), PushLit.into());
        machine.set16(i.inc_by(2), 0x0304);

        machine.set8(i.inc(), PushLit.into());
        machine.set16(i.inc_by(2), 0x0506);

        machine.set8(i.inc(), MoveLitToReg.into());
        machine.set16(i.inc_by(2), 0x0708);
        machine.set8(i.inc(), R1.into());

        machine.set8(i.inc(), MoveLitToReg.into());
        machine.set16(i.inc_by(2), 0x090A);
        machine.set8(i.inc(), R4.into());

        machine.set8(i.inc(), Ret.into());

        machine.set8(i.inc(), PushLit.into());
        machine.set16(i.inc_by(2), 0x9999);

    }

    #[test]
    fn load_machine() {
        let mut machine = Machine::default();

        // let mut i = Ptr(0);
        println!("\nInitial Machine State:");

        print_machine_state(&machine, &[]);

        counter_program(&mut machine);
        
        // swap_registers_program(&mut machine);

        // stack_frame_program(&mut machine);
        
        println!("\nLoaded Instructions:");

        print_machine_state(&machine, &[
            ("SUB".to_owned(), Ptr(0x3000), 32)
        ]);

        println!("\nStepping Program:");

        for _ in 0..20 {
            machine.step().unwrap();

            print_machine_state(&machine, &[
                ("SUB".to_owned(), Ptr(0x3000), 32)
            ]);
        }

        panic!("Ended Program on Purpose!");
    }
}
