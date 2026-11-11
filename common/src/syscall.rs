use core::arch::asm;

#[repr(u64)]
#[derive(Debug)]
pub enum SyscallNumber {
    Out = 1,
    Fork = 2,
    Exec = 3,
    Exit = 4,
    MMap = 5,
    MUnmap = 6,
    Open = 7,
    Read = 8,
    Write = 9,
    Close = 10,
    DuplicateFd = 11,
    ReplaceFd = 12,
}

pub fn syscall(syscall_number: SyscallNumber, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let mut result;
    unsafe {
        asm!(
            "int 0x80",
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("rcx") arg3,
            in("r8")  syscall_number as u64,
            lateout("rax") result,
            clobber_abi("C"),
        );
    }
    result
}
