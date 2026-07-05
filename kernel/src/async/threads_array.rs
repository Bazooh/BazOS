use crate::r#async::thread::Thread;
use alloc::sync::Arc;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::Ordering::Relaxed;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use spin::Mutex;
use std::serial_println;

pub struct ThreadsArray {
    array: Arc<Mutex<ThreadsArrayInternal>>,
}

struct ThreadsArrayInternal {
    threads: [Option<Thread>; 128],
    fill_mask: u128,
    available_mask: u128,
}

pub struct ThreadRef {
    thread: &'static mut Thread,
    array: Arc<Mutex<ThreadsArrayInternal>>,
    index: u32,
}

impl ThreadsArray {
    pub fn new() -> Self {
        ThreadsArray {
            array: Arc::new(Mutex::new(unsafe {
                MaybeUninit::<ThreadsArrayInternal>::zeroed().assume_init()
            })),
        }
    }

    pub fn add(&self, thread: Thread) -> Option<()> {
        let mut array = self.array.lock();

        let index = array.fill_mask.trailing_ones();
        if index == 128 {
            return None;
        }
        serial_println!("add {:?} at {:?}", thread, index);

        array.threads[index as usize] = Some(thread);
        array.fill_mask |= 1u128 << index;
        array.available_mask |= 1u128 << index;
        Some(())
    }

    pub fn pick(&self) -> Option<ThreadRef> {
        let mut array = self.array.lock();

        // TODO choose better
        static LAST_INDEX: AtomicU32 = AtomicU32::new(0);

        let index = loop {
            let index = LAST_INDEX
                .try_update(Relaxed, Relaxed, |x| Some((x + 1) % 128))
                .unwrap();
            if array.available_mask & (1u128 << index) != 0 {
                break index;
            }
        };

        serial_println!("pick {:?}", index);

        let mask = 1 << index;

        assert_eq!(array.fill_mask & mask, mask);
        assert_eq!(array.available_mask & mask, mask);

        array.available_mask &= !mask;
        let ptr = array.threads[index as usize].as_mut().unwrap() as *mut Thread;
        Some(ThreadRef {
            thread: unsafe { &mut *ptr },
            array: Arc::clone(&self.array),
            index,
        })
    }
}

impl ThreadsArrayInternal {
    fn make_available(&mut self, index: u32) {
        let mask = 1u128 << index;

        assert_ne!(self.fill_mask & mask, 0);
        assert_eq!(self.available_mask & mask, 0);

        self.available_mask |= mask;
    }
}

impl ThreadRef {
    fn make_available(&mut self) {
        self.array.lock().make_available(self.index);
    }
}

impl Deref for ThreadRef {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        self.thread
    }
}

impl DerefMut for ThreadRef {
    fn deref_mut(&mut self) -> &mut Thread {
        self.thread
    }
}

impl Drop for ThreadRef {
    fn drop(&mut self) {
        self.make_available();
    }
}
