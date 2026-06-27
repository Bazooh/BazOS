use x86_64::{
    VirtAddr,
    structures::paging::{PageTableFlags, Size4KiB, mapper::MapToError},
};

use super::{CompositeAllocator, HEAP_SIZE, HEAP_START};
use crate::memory::memory_mapper::PageMapper;
use crate::memory::{PAGE_SIZE, buddy_allocator::compute_max_depth};

const HEAP_MAX_DEPTH: usize = compute_max_depth(HEAP_SIZE, PAGE_SIZE);

#[global_allocator]
pub static HEAP: CompositeAllocator<HEAP_MAX_DEPTH> = CompositeAllocator::new();

pub fn init_heap(page_mapper: &mut PageMapper) -> Result<(), MapToError<Size4KiB>> {
    page_mapper.map(
        VirtAddr::new(HEAP_START as u64),
        HEAP_SIZE,
        PageTableFlags::WRITABLE,
    )?;

    HEAP.init();

    Ok(())
}
