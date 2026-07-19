use core::arch::asm;

use crate::r#async::threads_array::ThreadRef;
use crate::r#async::threads_array::ThreadsArray;
use crate::r#async::{process::Process, thread::Thread};
use alloc::sync::Arc;
use alloc::{boxed::Box, collections::btree_map::BTreeMap};
use conquer_once::spin::OnceCell;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::{Mutex, MutexGuard};

static SCHEDULER: OnceCell<Mutex<Scheduler>> = OnceCell::uninit();

static LAST_PID: AtomicU64 = AtomicU64::new(0);

pub struct Scheduler {
    processes: BTreeMap<u64, Arc<Mutex<Process>>>,
    threads: ThreadsArray,
}

impl Scheduler {
    fn new() -> Scheduler {
        Scheduler {
            processes: BTreeMap::new(),
            threads: ThreadsArray::new(),
        }
    }

    pub fn lock<'a>() -> MutexGuard<'a, Scheduler> {
        // TODO: This should probably disable interrupts to avoid deadlock
        SCHEDULER.get().expect("Scheduler not initialized").lock()
    }

    pub fn next_pid() -> u64 {
        LAST_PID.fetch_add(1, Ordering::Relaxed)
    }

    pub fn add_process(&mut self, process: Process) {
        self.processes
            .insert(process.pid(), Arc::new(Mutex::new(process)));
    }

    pub fn get_process(&mut self, pid: u64) -> Option<Arc<Mutex<Process>>> {
        Some(Arc::clone(self.processes.get(&pid)?))
    }

    pub fn add_thread(&mut self, thread: Thread) {
        self.threads
            .add(Arc::new(Mutex::new(thread)))
            .expect("Max threads exceeded");
    }

    pub fn pick_thread(&self) -> Option<ThreadRef> {
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
