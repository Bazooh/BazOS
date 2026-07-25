use common::syscall::{SyscallNumber::Fork, syscall};

#[derive(Debug)]
pub struct Pid(pub u64);

impl Pid {
    pub const fn root() -> Self {
        Self(0)
    }

    pub fn is_root(&self) -> bool {
        self.0 == 0
    }

    pub fn pid(&self) -> u64 {
        self.0
    }
}

pub fn fork() -> Option<Pid> {
    match syscall(Fork, 0, 0, 0, 0) {
        0 => None,
        pid if pid > 0 => Some(Pid(pid as u64)),
        _ => unreachable!(),
    }
}
