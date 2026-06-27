use crate::syscall::{SyscallNumber, syscall};

#[derive(Debug)]
pub struct Pid(usize);

impl Pid {
    pub const fn root() -> Self {
        Self(0)
    }

    pub fn is_root(&self) -> bool {
        self.0 == 0
    }

    pub fn pid(&self) -> usize {
        self.0
    }
}

pub fn fork() -> Option<Pid> {
    match syscall(SyscallNumber::Fork, 0, 0, 0) {
        0 => None,
        pid if pid > 0 => Some(Pid(pid as usize)),
        _ => unreachable!(),
    }
}
