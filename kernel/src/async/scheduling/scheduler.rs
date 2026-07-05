use core::arch::asm;

use crate::r#async::threads_array::ThreadRef;
use crate::r#async::threads_array::ThreadsArray;
use crate::r#async::{process::Process, thread::Thread};
use alloc::{boxed::Box, collections::btree_map::BTreeMap};
use conquer_once::spin::OnceCell;
use spin::{Mutex, MutexGuard};

static SCHEDULER: OnceCell<Mutex<Scheduler>> = OnceCell::uninit();

pub struct Scheduler {
    processes: BTreeMap<u64, Process>,
    threads: ThreadsArray,
    last_pid: u64,
}

impl Scheduler {
    fn new() -> Scheduler {
        Scheduler {
            processes: BTreeMap::new(),
            last_pid: 0,
            threads: ThreadsArray::new(),
        }
    }

    pub fn lock<'a>() -> MutexGuard<'a, Scheduler> {
        SCHEDULER.get().expect("Scheduler not initialized").lock()
    }

    pub fn next_pid(&mut self) -> u64 {
        self.last_pid += 1;
        self.last_pid
    }

    pub fn add_process(&mut self, process: Process) {
        self.processes.insert(process.pid(), process);
    }

    pub fn get_process(&self, pid: u64) -> Option<&Process> {
        self.processes.get(&pid)
    }

    pub fn get_process_mut(&mut self, pid: u64) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    pub fn add_thread(&mut self, thread: Thread) {
        self.threads.add(thread).expect("Max threads exceeded");
    }

    pub fn pick_thread(&mut self) -> Option<ThreadRef> {
        self.threads.pick()
    }
}

pub fn init_scheduler() {
    SCHEDULER
        .try_init_once(|| Mutex::new(Scheduler::new()))
        .expect("Scheduler already initialized");

    const MSR_GS_BASE: u32 = 0xC0000101;
    let addr = &(*Box::new(0 as *mut Thread)) as *const _ as u64;
    let low = addr as u32;
    let high = (addr >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") MSR_GS_BASE,
            in("eax") low,
            in("edx") high,
        )
    };
}
