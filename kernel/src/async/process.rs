use core::mem::MaybeUninit;

use alloc::boxed::Box;

use crate::r#async::{scheduler::Scheduler, thread::Thread};

pub struct Process {
    pid: u32,
    parent_pid: u32,
    name: &'static str,
    main_thread: Thread,

    code: *const (),
    data: *const (),
    stack: *const (),
}

pub const STACK_SIZE: usize = 4096 * 5;

impl Process {
    fn new(name: &'static str, parent_pid: u32, entry_point: fn()) -> Process {
        let pid = Scheduler::next_pid();
        let main_thread = Thread::new(entry_point);

        let stack: Box<[MaybeUninit<u8>; STACK_SIZE]> =
            Box::new([MaybeUninit::uninit(); STACK_SIZE]);

        Process {
            pid,
            parent_pid,
            name,
            main_thread,
            code: 0 as *const (),
            data: 0 as *const (),
            stack: stack.as_ptr() as *const (),
        }
    }

    pub fn create(name: &'static str, parent_pid: u32, entry_point: fn()) {
        let process = Process::new(name, parent_pid, entry_point);
        // Scheduler::add_process(process);
    }
}
