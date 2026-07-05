use crate::memory::{
    HEAP_SIZE, HEAP_START, PAGE_SIZE, binary_allocator::BinaryAllocator,
    buddy_allocator::BuddyAllocator, slab_allocator::SlabAllocator,
};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
};
use spin::Mutex;

pub struct KernelAllocator<const MAX_DEPTH: usize> {
    slab_allocator: Mutex<SlabAllocator<BuddyAllocator<MAX_DEPTH>>>,
}

impl<const MAX_DEPTH: usize> KernelAllocator<MAX_DEPTH> {
    pub const fn new() -> Self {
        KernelAllocator {
            slab_allocator: Mutex::new(SlabAllocator::new()),
        }
    }

    pub fn init(&self) {
        let mut buddy_allocator =
            BuddyAllocator::<MAX_DEPTH>::new(HEAP_SIZE, PAGE_SIZE, HEAP_START);
        buddy_allocator.init();
        self.slab_allocator.lock().init(buddy_allocator);
    }
}

unsafe impl<const MAX_DEPTH: usize> GlobalAlloc for KernelAllocator<MAX_DEPTH> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = self.slab_allocator.lock().compute_size(layout);
        self.slab_allocator
            .lock()
            .alloc(size)
            .unwrap_or_else(|| null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = self.slab_allocator.lock().compute_size(layout);
        self.slab_allocator.lock().dealloc(ptr, size);
    }
}
