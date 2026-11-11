use crate::r#async::thread::ThreadId;
use crate::r#async::threads_array::ThreadRef;
use crate::interrupts;
use core::cell::{Ref, RefCell, RefMut};
use core::mem;

pub struct Worker {
    current_thread: Option<ThreadRef>,
    /// Thread picked by the timer handler, consumed by the context switch
    next_thread: Option<ThreadRef>,
}

struct CurrentWorker {
    worker: RefCell<Worker>,
}

unsafe impl Send for CurrentWorker {}
unsafe impl Sync for CurrentWorker {}

static CURRENT: CurrentWorker = CurrentWorker {
    worker: RefCell::new(Worker {
        current_thread: None,
        next_thread: None,
    }),
};

impl Worker {
    pub fn current<'a>() -> Ref<'a, Worker> {
        CURRENT.worker.borrow()
    }

    pub fn current_mut<'a>() -> RefMut<'a, Worker> {
        CURRENT.worker.borrow_mut()
    }

    pub fn swap_thread(&mut self, new_thread: Option<ThreadRef>) -> Option<ThreadRef> {
        mem::replace(&mut self.current_thread, new_thread)
    }

    pub fn set_next_thread(&mut self, thread: ThreadRef) {
        self.next_thread = Some(thread);
    }

    pub fn take_next_thread(&mut self) -> Option<ThreadRef> {
        self.next_thread.take()
    }

    pub fn current_thread_id(&self) -> Option<ThreadId> {
        Some(self.current_thread.as_ref()?.id())
    }

    pub fn run() -> ! {
        interrupts::enable();
        loop {} // Context switch will run threads
    }
}
