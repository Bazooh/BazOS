use common::syscall::{SyscallNumber::Out, syscall};
use core::fmt::{Result, Write};

pub struct Writer;

impl Write for Writer {
    fn write_str(&mut self, string: &str) -> Result {
        match syscall(Out, string.as_ptr() as u64, string.len() as u64, 0, 0) {
            0 => Ok(()),
            _ => Err(core::fmt::Error),
        }
    }
}
