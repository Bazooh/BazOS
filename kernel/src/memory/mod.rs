use bootloader::bootinfo::MemoryMap;
use spin::Mutex;

use crate::memory::bootloader_allocator::{BootLoaderFrameAllocator, FRAME_ALLOCATOR};

use crate::memory::memory_mapper::MemoryMapper;
pub use bootloader_allocator::to_virtual_address;
pub use composite_allocator::KernelAllocator;
pub use heap::KERNEL_ALLOCATOR;
pub use program_allocator::PROGRAM_ALLOCATOR;

pub mod allocator;
mod binary_allocator;
mod bootloader_allocator;
mod buddy_allocator;
mod composite_allocator;
mod heap;
pub mod memory_mapper;
mod program_allocator;
mod slab_allocator;

pub const HEAP_START: u64 = 0x2000_0000_0000;
pub const HEAP_SIZE: u64 = 1024 * 1024; // 1 MiB
pub const PAGE_SIZE: u64 = 4096;

pub const PROGRAM_HEAP_START: u64 = 0xffffa00000000000;

pub fn init_memory(physical_memory_offset: u64, memory_map: &'static MemoryMap) {
    FRAME_ALLOCATOR
        .try_init_once(move || Mutex::new(unsafe { BootLoaderFrameAllocator::new(memory_map) }))
        .expect("Frame allocator already initialized");

    unsafe {
        MemoryMapper::init(physical_memory_offset);
    }
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
