use crate::Ptr;

use core::fmt;

pub struct MemoryWindow<'a> {
    pub(crate) addr: Ptr,
    pub(crate) data: &'a [u8],
}

impl<'a> MemoryWindow<'a> {
    pub fn new(addr: Ptr, data: &'a [u8]) -> Self {
        MemoryWindow { addr, data }
    }

    pub fn ptr(&self) -> Ptr {
        self.addr
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

impl<'a> fmt::Debug for MemoryWindow<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Emit the initial address of the region
        write!(f, "[{:?}] ", self.addr)?;
        let len = self.data.len();
        let mut i = 0;
        while i < len {
            if self.data[i] == 0 {
                // Emit dashes
                write!(f, "------")?;
            } else {
                // Emit the value at the current position
                write!(f, "{:#04X}", self.data[i])?;
            }
            i += 1;
            if i == len {
                // return Ok(());
                break;
            }
            // Emit next block address or spacer for values
            if i % 8 == 0 {
                write!(f, "\n[{:?}] ", self.addr + i)?;
            } else {
                write!(f, " ")?;
            }
        }
        while i % 8 != 0 {
            // Finish the last line out with placeholders
            write!(f, " _x__")?;
            i += 1;
        }
        Ok(())
    }
}
