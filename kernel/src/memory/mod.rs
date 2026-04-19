use bootloader::bootinfo::MemoryMap;
use spin::Mutex;

use crate::memory::{
    frame_allocator::{BootLoaderFrameAllocator, FRAME_ALLOCATOR, init_memory_mapper},
    heap::init_heap,
    program_allocator::init_program_allocator,
};

pub use composite_allocator::CompositeAllocator;
pub use frame_allocator::{MEMORY_MAPPER, to_virtual_address};
pub use heap::HEAP;
pub use program_allocator::PROGRAM_ALLOCATOR;

mod binary_allocator;
mod buddy_allocator;
mod composite_allocator;
mod frame_allocator;
mod heap;
mod program_allocator;
mod slab_allocator;

pub const HEAP_START: usize = 0x4444_0000_0000;
pub const HEAP_SIZE: usize = 128 * 1024; // 128 KiB
pub const PAGE_SIZE: usize = 4096;

pub fn init_memory(physical_memory_offset: u64, memory_map: &'static MemoryMap) {
    FRAME_ALLOCATOR
        .try_init_once(move || Mutex::new(unsafe { BootLoaderFrameAllocator::new(memory_map) }))
        .expect("Frame allocator already initialized");

    let mut memory_mapper = unsafe { init_memory_mapper(physical_memory_offset) };

    init_heap(&mut memory_mapper).expect("Heap initialization failed");
    init_program_allocator(&mut memory_mapper).expect("Program allocator initialization failed");

    MEMORY_MAPPER
        .try_init_once(move || memory_mapper)
        .expect("Memory mapper already initialized");
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
