use crate::interrupts::{
    idt::ExceptionStackFrame,
    syscall::{fork::fork_handler, out::out_handler},
};

mod fork;
mod out;

#[repr(u64)]
#[derive(Debug)]
pub enum SyscallNumber {
    Out = 1,
    Fork = 2,
}

pub extern "C" fn syscall_handler(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    syscall_number: SyscallNumber,
    frame: ExceptionStackFrame,
) -> isize {
    match syscall_number {
        SyscallNumber::Out => out_handler(arg0 as *const u8, arg1),
        SyscallNumber::Fork => fork_handler(),
    }
}
