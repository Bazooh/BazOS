use crate::interrupts::{
    idt::ExceptionStackFrame,
    syscall::{fork::fork_handler, out::out_handler},
};
use core::arch::asm;
use std::serial_println;

mod fork;
mod out;

#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum SyscallNumber {
    Out = 1,
    Fork = 2,
}

pub extern "C" fn syscall_handler(
    arg0: usize,
    arg1: usize,
    _arg2: usize,
    syscall_number: SyscallNumber,
    frame: &ExceptionStackFrame,
) -> isize {
    serial_println!("Syscall: {:?}", syscall_number);
    let rsp = unsafe {
        let mut rsp: u64;
        asm!("mov {rsp}, rsp", rsp = out(reg) rsp);
        rsp
    };
    serial_println!("RSP: {:x}", rsp);
    match syscall_number {
        SyscallNumber::Out => out_handler(arg0 as *const u8, arg1),
        SyscallNumber::Fork => fork_handler(frame),
    }
}
