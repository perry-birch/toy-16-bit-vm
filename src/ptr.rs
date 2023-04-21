use core::fmt;
use core::ops::*;

use derive_more::*;

#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    Sub,
    BitAnd,
    BitOr,
    BitXor,
    Mul,
    Div,
    Rem,
    Shr,
    Shl,
    AddAssign,
    SubAssign,
    From,
    Into,
    Deref,
    DerefMut,
    AsRef,
    AsMut,
    Display,
)]
pub struct Ptr(pub u16);

impl Ptr {
    /// Increments the pointer by one and returns
    /// a new pointer with the prior value
    /// Used in lieu of ptr++
    pub fn inc(&mut self) -> Ptr {
        self.inc_by(1)
    }

    /// Increments the pointer by the desired amount and returns
    /// a new pointer with the prior value
    /// Used in lieu of ptr++ x N
    pub fn inc_by<T>(&mut self, v: T) -> Ptr
    where
        Self: Add<T, Output = Self>,
    {
        let curr = self.0;
        // TODO: Bounds check needed?
        (*self) = (*self) + v;
        Ptr(curr) // Return the previous position
    }
}

macro_rules! add_to_ptr {
    ($($type:ty),*) => {$(
        impl Add<$type> for Ptr {
            type Output = Ptr;
        
            fn add(self, other: $type) -> Ptr {
                Ptr(self.0 + other as u16)
            }
        }
    )*}
}

add_to_ptr!(i8, u8, i16, u16, i32, u32, i64, u64, usize);

impl fmt::LowerHex for Ptr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#06x}", self.0))
    }
}

impl fmt::UpperHex for Ptr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#06X}", self.0))
    }
}

impl fmt::Debug for Ptr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#06X}", self.0))
    }
}
