use core::sync::atomic::{AtomicU32, Ordering};

use alloc::vec::Vec;

use crate::{
    r#async::{process::Process, thread::Thread},
    utils::tree::Tree,
};

pub struct Scheduler {
    running_threads: Vec<Thread>,
    waiting_threads: Vec<Thread>,
    process_tree: Tree<Process>,
}

impl Scheduler {
    pub fn next_pid() -> u32 {
        static PID: AtomicU32 = AtomicU32::new(0);
        PID.fetch_add(1, Ordering::SeqCst)
    }
}
