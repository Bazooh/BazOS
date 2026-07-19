use crate::r#async::thread::ThreadId;
use crate::r#async::threads_array::ThreadRef;
use crate::interrupts;
use alloc::boxed::Box;
use core::cell::{Ref, RefCell, RefMut};
use core::mem;

pub struct Worker {
    current_thread: Option<Box<ThreadRef>>,
}

struct CurrentWorker {
    worker: RefCell<Worker>,
}

unsafe impl Send for CurrentWorker {}
unsafe impl Sync for CurrentWorker {}

static CURRENT: CurrentWorker = CurrentWorker {
    worker: RefCell::new(Worker {
        current_thread: None,
    }),
};

impl Worker {
    pub fn current<'a>() -> Ref<'a, Worker> {
        CURRENT.worker.borrow()
    }

    pub fn current_mut<'a>() -> RefMut<'a, Worker> {
        CURRENT.worker.borrow_mut()
    }

    pub fn swap_thread(&mut self, new_thread: Option<Box<ThreadRef>>) -> Option<Box<ThreadRef>> {
        mem::replace(&mut self.current_thread, new_thread)
    }

    pub fn current_thread_id(&self) -> Option<ThreadId> {
        Some(self.current_thread.as_ref()?.id())
    }

    pub fn run() -> ! {
        interrupts::enable();
        loop {} // Context switch will run threads
    }
}
