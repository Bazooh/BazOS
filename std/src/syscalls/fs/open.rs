use crate::syscalls::fs::file_descriptor::{FileDescriptor, FileDescriptorResource};
use common::syscall::SyscallNumber::Open;
use common::syscall::syscall;
use common::syscall_errors::open::OpenError;

pub fn open(filename: &str) -> Result<impl FileDescriptor, OpenError> {
    let index = syscall(Open, filename.as_ptr() as u64, filename.len() as u64, 0, 0);
    if index < 0 {
        return Err(OpenError::try_from(index).expect("Unknown error"));
    }

    Ok(FileDescriptorResource::new(filename, index as u64))
}
