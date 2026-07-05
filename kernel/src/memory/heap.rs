use x86_64::{
    VirtAddr,
    structures::paging::{PageTableFlags, Size4KiB, mapper::MapToError},
};

use super::{HEAP_SIZE, HEAP_START, KernelAllocator};
use crate::memory::memory_mapper::PageMapper;
use crate::memory::{PAGE_SIZE, buddy_allocator::compute_max_depth};

const HEAP_MAX_DEPTH: usize = compute_max_depth(HEAP_SIZE, PAGE_SIZE) as usize;

#[global_allocator]
pub static KERNEL_ALLOCATOR: KernelAllocator<HEAP_MAX_DEPTH> = KernelAllocator::new();

pub fn init_heap(page_mapper: &mut PageMapper) -> Result<(), MapToError<Size4KiB>> {
    page_mapper.map(
        VirtAddr::new(HEAP_START),
        HEAP_SIZE,
        PageTableFlags::WRITABLE,
    )?;

    KERNEL_ALLOCATOR.init();

    Ok(())
}
