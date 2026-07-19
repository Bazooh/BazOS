use crate::r#async::thread::Thread;
use crate::r#async::thread::ThreadId;
use r#alloc::sync::Arc;
use bit_field::BitField;
use core::ops::Deref;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;
use spin::{Mutex, RwLock};
use x86_64::{PhysAddr, VirtAddr};

pub struct ThreadsArray {
    array: Arc<RwLock<ThreadsArrayInternal>>,
}

struct ThreadsArrayInternal {
    threads: [Option<Arc<Mutex<Thread>>>; 128],
    fill_mask: u128,
    available_mask: u128,
    should_die_mask: u128,
}

pub struct ThreadRef {
    pub thread: Arc<Mutex<Thread>>,
    array: Arc<RwLock<ThreadsArrayInternal>>,
    index: usize,
}

impl ThreadsArray {
    pub fn new() -> Self {
        const NOTHING: Option<Arc<Mutex<Thread>>> = None;
        ThreadsArray {
            array: Arc::new(RwLock::new(ThreadsArrayInternal {
                threads: [NOTHING; 128],
                fill_mask: 0,
                available_mask: 0,
                should_die_mask: 0,
            })),
        }
    }

    pub fn add(&self, thread: Arc<Mutex<Thread>>) -> Option<()> {
        let mut array = self.array.write();

        let index = array.fill_mask.trailing_ones() as usize;
        if index == 128 {
            return None;
        }

        array.threads[index] = Some(thread);
        array.fill_mask.set_bit(index, true);
        array.available_mask.set_bit(index, true);
        Some(())
    }

    pub fn pick(&self) -> Option<ThreadRef> {
        let mut array = self.array.write();

        // TODO choose better + Does not work with only one thread
        static LAST_INDEX: AtomicU32 = AtomicU32::new(0);

        let index = loop {
            let index = LAST_INDEX
                .try_update(Relaxed, Relaxed, |x| Some((x + 1) % 128))
                .unwrap() as usize;
            if array.available_mask.get_bit(index) {
                break index;
            }
        };

        assert!(array.fill_mask.get_bit(index));
        assert!(array.available_mask.get_bit(index));

        array.available_mask.set_bit(index, false);
        Some(ThreadRef {
            thread: Arc::clone(array.threads[index].as_ref().unwrap()),
            array: Arc::clone(&self.array),
            index,
        })
    }

    pub fn kill(&self, filter: impl Fn(ThreadId) -> bool) {
        let mut mask = 0;
        for (index, thread) in self.array.read().threads.iter().enumerate() {
            if let Some(thread) = thread
                && filter(thread.lock().id())
            {
                mask.set_bit(index, true);
            }
        }
        self.array.write().should_die_mask |= mask;
    }
}

impl ThreadsArrayInternal {
    fn make_available(&mut self, index: usize) {
        assert!(self.fill_mask.get_bit(index));
        assert!(!self.available_mask.get_bit(index));

        self.available_mask.set_bit(index, true);
    }

    fn remove(&mut self, index: usize) -> Arc<Mutex<Thread>> {
        assert!(self.fill_mask.get_bit(index));

        self.fill_mask.set_bit(index, false);

        self.threads[index].take().unwrap()
    }
}

impl ThreadRef {
    pub fn make_available(&self) {
        self.array.write().make_available(self.index);
    }

    pub fn remove(self) -> Arc<Mutex<Thread>> {
        self.array.write().remove(self.index)
    }

    pub fn should_die(&self) -> bool {
        self.array.read().should_die_mask.get_bit(self.index)
    }

    pub fn as_ptr(&self) -> *const Mutex<Thread> {
        self.thread.deref()
    }

    pub fn stack_pointer(&self) -> VirtAddr {
        self.thread.lock().stack_pointer()
    }

    pub fn kernel_stack_pointer(&self) -> VirtAddr {
        self.thread.lock().kernel_stack_pointer()
    }

    pub fn page_table_address(&self) -> PhysAddr {
        self.thread.lock().page_table_addr()
    }

    pub fn set_stack_pointer(&self, stack_pointer: VirtAddr) {
        self.thread.lock().stack_pointer = stack_pointer;
    }

    pub fn id(&self) -> ThreadId {
        self.thread.lock().id()
    }
}

impl Drop for ThreadRef {
    fn drop(&mut self) {
        self.make_available();
    }
}
