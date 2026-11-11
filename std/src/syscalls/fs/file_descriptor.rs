use alloc::string::String;
use common::fs::FileDescriptorIndex;
use common::opened_resource::{BLOCK_SIZE, Resource};
use common::syscall::SyscallNumber::{Close, DuplicateFd, Read, ReplaceFd, Write};
use common::syscall::syscall;
use common::syscall_errors::duplicate_fd::DuplicateFdError;
use common::syscall_errors::read::ReadError;
use common::syscall_errors::replace_fd::ReplaceFdError;

pub(crate) struct FileDescriptorResource {
    name: String,
    fd_index: FileDescriptorIndex,
}

pub trait FileDescriptor: Resource {
    fn index(&self) -> FileDescriptorIndex;

    fn duplicate(&self) -> Result<impl FileDescriptor + use<Self>, DuplicateFdError>;

    /// Makes `fd_index` refer to the same resource as `self`.
    fn replace(&self, fd_index: FileDescriptorIndex) -> Result<(), ReplaceFdError>;
}

impl FileDescriptorResource {
    pub(crate) fn new(name: &str, fd_index: u64) -> Self {
        Self {
            name: String::from(name),
            fd_index: FileDescriptorIndex::new(fd_index),
        }
    }
}

impl FileDescriptor for FileDescriptorResource {
    fn index(&self) -> FileDescriptorIndex {
        self.fd_index
    }

    fn duplicate(&self) -> Result<impl FileDescriptor + use<>, DuplicateFdError> {
        let fd_index = syscall(DuplicateFd, self.fd_index.as_u64(), 0, 0, 0);
        if fd_index < 0 {
            return Err(DuplicateFdError::try_from(fd_index).expect("Unknown error"));
        }
        Ok(Self::new(self.name(), fd_index as u64))
    }

    fn replace(&self, fd_index: FileDescriptorIndex) -> Result<(), ReplaceFdError> {
        let result = syscall(ReplaceFd, fd_index.as_u64(), self.fd_index.as_u64(), 0, 0);
        if result < 0 {
            return Err(ReplaceFdError::try_from(result).expect("Unknown error"));
        }
        Ok(())
    }
}

impl Resource for FileDescriptorResource {
    fn read_into(&self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        let read = syscall(
            Read,
            self.fd_index.as_u64(),
            buffer.as_mut_ptr() as u64,
            buffer.len() as u64,
            0,
        );
        match read {
            length if length >= 0 => Ok(length as usize),
            _ => Err(ReadError::try_from(read).unwrap()),
        }
    }

    fn write(&self, data: &[u8]) -> Result<(), ()> {
        let wrote = syscall(
            Write,
            self.fd_index.as_u64(),
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
        syscall(Close, self.fd_index.as_u64(), 0, 0, 0);
    }
}
