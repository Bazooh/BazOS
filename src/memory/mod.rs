use bootloader::bootinfo::MemoryMap;

use crate::memory::heap::init_heap;

pub use composite_allocator::CompositeAllocator;

mod binary_allocator;
mod buddy_allocator;
mod composite_allocator;
mod frame_allocator;
mod heap;
mod slab_allocator;

pub const HEAP_START: usize = 0x_4444_0000_0000;
pub const HEAP_SIZE: usize = 128 * 1024; // 128 KiB
pub const PAGE_SIZE: usize = 4096;

pub fn init_memory(physical_memory_offset: u64, memory_map: &'static MemoryMap) {
    init_heap(physical_memory_offset, memory_map).expect("Heap initialization failed");
}

#[repr(C)]
struct FreeSpaceNode {
    next: Option<&'static mut FreeSpaceNode>,
}

impl FreeSpaceNode {
    fn new() -> FreeSpaceNode {
        FreeSpaceNode { next: None }
    }
}
