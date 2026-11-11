use crate::syscalls::fs::file_descriptor_resource::FileDescriptorResource;
use alloc::string::String;
use common::fs::FileDescriptor;
use common::opened_resource::OpenedResource;
use common::syscall::SyscallNumber::Open;
use common::syscall::syscall;
use common::syscall_errors::open::OpenError;

pub fn open(filename: &str) -> Result<impl OpenedResource, OpenError> {
    let index = syscall(Open, filename.as_ptr() as u64, filename.len() as u64, 0, 0);
    if index < 0 {
        return Err(OpenError::try_from(index).expect("Unknown error"));
    }

    Ok(FileDescriptorResource::new(
        String::from(filename),
        FileDescriptor::new(index as u64),
    ))
}
