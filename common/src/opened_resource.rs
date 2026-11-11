use crate::syscall_errors::read::ReadError;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub const BLOCK_SIZE: usize = 512;

pub trait Resource: Send {
    fn read_all(&mut self) -> Result<Option<Box<[u8]>>, ReadError> {
        let mut content = Vec::new();
        let mut buf = [0u8; BLOCK_SIZE];
        while let size = self.read_into(&mut buf)?
            && size > 0
        {
            content.extend_from_slice(&buf[..size]);
        }
        if content.is_empty() {
            return Ok(None);
        }
        Ok(Some(content.into_boxed_slice()))
    }

    fn read_into(&mut self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError>;

    fn write(&mut self, data: &[u8]) -> Result<(), ()>;

    fn name(&self) -> &str;
}
