use alloc::string::String;
use common::fs::FileDescriptor;
use common::opened_resource::{BLOCK_SIZE, OpenedResource};
use common::syscall::SyscallNumber::{Close, Read, Write};
use common::syscall::syscall;
use common::syscall_errors::read::ReadError;

pub struct FileDescriptorResource {
    name: String,
    file_descriptor: FileDescriptor,
}

impl FileDescriptorResource {
    pub(crate) const fn new(name: String, file_descriptor: FileDescriptor) -> Self {
        Self {
            name,
            file_descriptor,
        }
    }
}

impl OpenedResource for FileDescriptorResource {
    fn read_into(&mut self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        let read = syscall(
            Read,
            self.file_descriptor.as_u64(),
            buffer.as_mut_ptr() as u64,
            buffer.len() as u64,
            0,
        );
        match read {
            length if length >= 0 => Ok(length as usize),
            _ => Err(ReadError::try_from(read).unwrap()),
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        let wrote = syscall(
            Write,
            self.file_descriptor.as_u64(),
            data.as_ptr() as u64,
            data.len() as u64,
            0,
        );
        if wrote != 0 {
            return Err(());
        }
        Ok(())
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Drop for FileDescriptorResource {
    fn drop(&mut self) {
        syscall(Close, self.file_descriptor.as_u64(), 0, 0, 0);
    }
}
