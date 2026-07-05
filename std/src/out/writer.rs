use crate::serial_println;
use crate::syscall::{SyscallNumber, syscall};
use core::fmt::{Result, Write};

pub struct Writer;

impl Write for Writer {
    fn write_str(&mut self, string: &str) -> Result {
        match syscall(
            SyscallNumber::Out,
            string.as_ptr() as usize,
            string.len(),
            0,
        ) {
            0 => Ok(()),
            _ => Err(core::fmt::Error),
        }
    }
}
