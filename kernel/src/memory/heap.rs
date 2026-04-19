use x86_64::{
    VirtAddr,
    structures::paging::{OffsetPageTable, PageTableFlags, Size4KiB, mapper::MapToError},
};

use crate::memory::{PAGE_SIZE, buddy_allocator::compute_max_depth, frame_allocator::map_pages};

use super::{CompositeAllocator, HEAP_SIZE, HEAP_START};

const HEAP_MAX_DEPTH: usize = compute_max_depth(HEAP_SIZE, PAGE_SIZE);

#[global_allocator]
pub static HEAP: CompositeAllocator<HEAP_MAX_DEPTH> = CompositeAllocator::new();

pub fn init_heap(memory_mapper: &mut OffsetPageTable<'_>) -> Result<(), MapToError<Size4KiB>> {
    map_pages(
        VirtAddr::new(HEAP_START as u64),
        HEAP_SIZE,
        PageTableFlags::WRITABLE,
        memory_mapper,
    )?;

    HEAP.init();

    Ok(())
}
