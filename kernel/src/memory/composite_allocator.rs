use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
};

use spin::Mutex;

use crate::memory::{
    binary_allocator::BinaryAllocator,
    buddy_allocator::BuddyAllocator,
    slab_allocator::{MINIMUM_BLOCK_SIZE, SlabAllocator},
};

pub struct CompositeAllocator {
    slab_allocator: Mutex<SlabAllocator<BuddyAllocator>>,
}

impl CompositeAllocator {
    pub const fn new() -> Self {
        CompositeAllocator {
            slab_allocator: Mutex::new(SlabAllocator::new()),
        }
    }

    pub fn init(&self) {
        let mut buddy_allocator = BuddyAllocator::new();
        buddy_allocator.init();
        self.slab_allocator.lock().init(buddy_allocator);
    }

    fn compute_size(layout: Layout) -> usize {
        layout
            .size()
            .max(layout.align())
            .max(MINIMUM_BLOCK_SIZE)
            .next_power_of_two()
    }
}

unsafe impl GlobalAlloc for CompositeAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = CompositeAllocator::compute_size(layout);
        match self.slab_allocator.lock().alloc(size) {
            Some(ptr) => ptr,
            None => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = CompositeAllocator::compute_size(layout);
        self.slab_allocator.lock().dealloc(ptr, size);
    }
}
