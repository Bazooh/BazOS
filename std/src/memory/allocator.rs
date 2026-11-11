use crate::syscalls::mmap::mmap;
use alloc::alloc::handle_alloc_error;
use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;
use spin::Mutex;

struct Allocator {
    allocator: Mutex<Option<LockedHeap>>,
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            allocator: Mutex::new(None),
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let address = unsafe {
            self.allocator
                .lock()
                .get_or_insert_with(|| {
                    let n_pages = 128;
                    let addr = mmap(n_pages).expect("Kernel is out of memory");
                    LockedHeap::new(addr.as_mut_ptr(), n_pages as usize * 4096)
                })
                .alloc(layout)
        };

        if address.is_null() {
            handle_alloc_error(layout);
        }
        address
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.allocator.lock().as_mut().unwrap().dealloc(ptr, layout) }
    }
}
