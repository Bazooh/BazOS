use crate::print;
use common::opened_resource::{BLOCK_SIZE, Resource};
use common::syscall_errors::read::ReadError;
use core::str::from_utf8;

pub struct Screen;

impl Resource for Screen {
    fn read_into(&self, _buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        Err(ReadError::OperationNotSupported)
    }

    fn write(&self, data: &[u8]) -> Result<(), ()> {
        let str = from_utf8(data).map_err(|_| ())?;
        print!("{str}");
        Ok(())
    }

    fn name(&self) -> &str {
        "screen.x"
    }
}
