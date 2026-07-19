use crate::syscall::SyscallNumber::Exit;
use crate::syscall::syscall;

pub fn exit(code: u64) -> ! {
    syscall(Exit, code, 0, 0, 0);
    unreachable!()
}
