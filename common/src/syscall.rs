use core::arch::asm;

#[repr(u64)]
#[derive(Debug)]
pub enum SyscallNumber {
    /// Out(ptr, len)
    Out = 1,
    /// Fork() -> pid
    Fork = 2,
    /// Exec(file_ptr, file_len, args_ptr, args_len) -> pid
    Exec = 3,
    /// Exit(code) -> !
    Exit = 4,
    /// MMap(n_pages) -> addr
    MMap = 5,
    /// MUnmap(addr, n_pages)
    MUnmap = 6,
    /// Open(filename_ptr,filename_len) -> fd
    Open = 7,
    /// Read(fd, buffer_ptr, buffer_len) -> n_bytes
    Read = 8,
    /// Write(fd, buffer_ptr, buffer_len) -> n_bytes
    Write = 9,
    /// Close(fd)
    Close = 10,
    /// DuplicateFd(fd) -> new_fd
    DuplicateFd = 11,
    /// ReplaceFd(old_fd, new_fd)
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
