use core::{
    fmt::{Debug, Formatter, LowerHex},
    ops::{Deref, DerefMut},
};

pub struct DebugHex<T>(T);

impl<T: LowerHex> Debug for DebugHex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "0x{:x}", self.0)
    }
}

impl Deref for DebugHex<u64> {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DebugHex<u64> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
