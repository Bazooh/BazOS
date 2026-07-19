use crate::interrupts::syscall::exec::exec_handler;
use crate::interrupts::{
    idt::ExceptionStackFrame,
    syscall::{fork::fork_handler, out::out_handler},
};
use crate::{eprintln, interrupts};
use core::slice;
use core::str::from_utf8;
use std::hlt_loop;

mod exec;
mod fork;
mod out;

#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum SyscallNumber {
    Out = 1,
    Fork = 2,
    Exec = 3,
    Exit = 4,
}

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
            slice::from_raw_parts(arg2 as *const &str, arg3 as usize)
        }),
        SyscallNumber::Exit => {
            // TODO: implement this
            interrupts::enable();
            hlt_loop();
        }
    }
}

unsafe fn to_str(ptr: u64, len: u64) -> Option<&'static str> {
    let result = from_utf8(unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) });
    match result {
        Ok(string) => Some(string),
        Err(err) => {
            // TODO: terminate the process ?
            eprintln!("{}", err);
            None
        }
    }
}
