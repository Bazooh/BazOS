use core::{
    arch::asm,
    sync::atomic::{AtomicU32, Ordering},
};
use std::{serial_print, serial_println};

use alloc::{boxed::Box, collections::btree_map::BTreeMap, vec::Vec};
use conquer_once::spin::OnceCell;
use spin::{Mutex, MutexGuard};

use crate::r#async::{process::Process, thread::Thread};

static SCHEDULER: OnceCell<Mutex<Scheduler>> = OnceCell::uninit();

pub struct Scheduler {
    processes: BTreeMap<u64, Process>,
    threads: Vec<Thread>,
    last_pid: u64,
}

impl Scheduler {
    fn new() -> Scheduler {
        Scheduler {
            processes: BTreeMap::new(),
            threads: Vec::new(),
            last_pid: 0,
        }
    }

    pub fn get<'a>() -> MutexGuard<'a, Scheduler> {
        SCHEDULER.get().expect("Scheduler not initialized").lock()
    }

    pub fn next_pid(&mut self) -> u64 {
        self.last_pid += 1;
        self.last_pid
    }

    pub fn add_thread(&mut self, thread: Thread) {
        self.threads.push(thread);
    }

    pub fn add_process(&mut self, process: Process) {
        self.add_thread(process.create_main_thread());
        self.processes.insert(process.pid(), process);
    }

    pub fn set_current_thread(thread: *mut Thread) {
        unsafe {
            asm!("mov gs:0, {}", in(reg) thread, options(nostack, preserves_flags));
        }
    }

    pub fn current_thread() -> *mut Thread {
        let thread;
        unsafe {
            asm!("mov {}, gs:0", out(reg) thread, options(nostack, preserves_flags));
        }
        thread
    }

    pub fn run(&mut self) -> ! {
        // TODO: Switch between threads
        let thread = self.threads.get(0).expect("No threads");
        thread.exec();
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
