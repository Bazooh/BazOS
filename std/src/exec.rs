use crate::fork::Pid;
use crate::syscall::{SyscallNumber, syscall};

pub fn exec(file_path: &str, args: &[&str]) -> Result<Pid, ()> {
    let arg0 = file_path.as_ptr() as u64;
    let arg1 = file_path.len() as u64;
    let arg2 = args.as_ptr() as u64;
    let arg3 = args.len() as u64;

    match syscall(SyscallNumber::Exec, arg0, arg1, arg2, arg3) {
        -1 => Err(()),
        pid if pid > 0 => Ok(Pid(pid as u64)),
        _ => unreachable!(),
    }
}
