use core::arch::asm;

#[repr(u64)]
#[derive(Debug)]
pub enum SyscallNumber {
    Out = 1,
    Fork = 2,
}

pub fn syscall(syscall_number: SyscallNumber, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let result;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") syscall_number as u64,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            lateout("rax") result,
            options(nostack)
        );
    }
    result
}
