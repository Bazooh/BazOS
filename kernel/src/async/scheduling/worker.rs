use crate::r#async::thread::Thread;
use crate::interrupts;
use core::arch::asm;

pub struct Worker {
    first_thread: Thread,
}

impl Worker {
    pub fn new() -> Self {
        Worker {
            first_thread: Thread::kernel(),
        }
    }

    pub fn set_current_thread(thread: &Thread) {
        unsafe {
            asm!("mov gs:0, {}", in(reg) thread);
        }
    }

    /// Safety: you must be the only one to use it
    pub unsafe fn current_thread() -> &'static mut Thread {
        let mut thread: *mut Thread;
        unsafe {
            asm!("mov {}, gs:0", out(reg) thread);
            &mut *thread
        }
    }

    pub fn run(self) -> ! {
        Self::set_current_thread(&self.first_thread);
        interrupts::enable();
        loop {} // Context switch will run the thread
    }
}
