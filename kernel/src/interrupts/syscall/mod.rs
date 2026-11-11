use crate::interrupts::syscall::close::close_handler;
use crate::interrupts::syscall::duplicate_fd::duplicate_fd_handler;
use crate::interrupts::syscall::exec::exec_handler;
use crate::interrupts::syscall::mmap::{mmap_handler, munmap_handler};
use crate::interrupts::syscall::open::open_handler;
use crate::interrupts::syscall::read::read_handler;
use crate::interrupts::syscall::replace_fd::replace_fd_handler;
use crate::interrupts::syscall::write::write_handler;
use crate::interrupts::{
    idt::ExceptionStackFrame,
    syscall::{fork::fork_handler, out::out_handler},
};
use crate::{hlt_loop, interrupts};
use common::fs::FileDescriptorIndex;
pub(crate) use common::syscall::SyscallNumber;
use core::slice;
use core::str::from_utf8;
use x86_64::VirtAddr;

mod exec;
mod fork;
mod mmap;
mod open;
mod out;

mod close;
mod duplicate_fd;
mod read;
mod replace_fd;
mod write;

pub unsafe extern "C" fn syscall_handler(
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    syscall_number: SyscallNumber,
    frame: &ExceptionStackFrame,
) -> i64 {
    match syscall_number {
        SyscallNumber::Out => out_handler(unsafe { to_str(arg0, arg1) }),
        SyscallNumber::Fork => fork_handler(frame),
        SyscallNumber::Exec => exec_handler(unsafe { to_str(arg0, arg1) }, unsafe {
            to_slice(arg2, arg3)
        }),
        SyscallNumber::Exit => {
            // TODO: implement this
            interrupts::enable();
            hlt_loop();
        }
        SyscallNumber::MMap => mmap_handler(arg0),
        SyscallNumber::MUnmap => munmap_handler(VirtAddr::new(arg0), arg1),
        SyscallNumber::Open => open_handler(unsafe { to_str(arg0, arg1) }),
        SyscallNumber::Read => read_handler(FileDescriptorIndex::new(arg0), unsafe {
            to_mut_slice(arg1, arg2)
        }),
        SyscallNumber::Write => write_handler(FileDescriptorIndex::new(arg0), unsafe {
            to_slice(arg1, arg2)
        }),
        SyscallNumber::Close => close_handler(FileDescriptorIndex::new(arg0)),
        SyscallNumber::DuplicateFd => duplicate_fd_handler(FileDescriptorIndex::new(arg0)),
        SyscallNumber::ReplaceFd => replace_fd_handler(
            FileDescriptorIndex::new(arg0),
            FileDescriptorIndex::new(arg1),
        ),
    }
}

unsafe fn to_str(ptr: u64, len: u64) -> Result<&'static str, ()> {
    from_utf8(unsafe { to_slice(ptr, len)? }).map_err(|_| ())
}

unsafe fn to_slice<T>(ptr: u64, len: u64) -> Result<&'static [T], ()> {
    Ok(unsafe { to_mut_slice(ptr, len)? })
}

unsafe fn to_mut_slice<T>(ptr: u64, len: u64) -> Result<&'static mut [T], ()> {
    // TODO: Check for safety, the user should not be able to read the kernel
    Ok(unsafe { slice::from_raw_parts_mut(ptr as *mut T, len as usize) })
}
