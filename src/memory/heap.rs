use bootloader::bootinfo::MemoryMap;
use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

use crate::{
    ALLOCATOR,
    memory::{frame_allocator::BootLoaderFrameAllocator, init_memory_mapper},
};

pub const HEAP_START: usize = 0x_4444_0000_0000;
pub const HEAP_SIZE: usize = 128 * 1024; // 128 KiB

pub fn init_heap(
    physical_memory_offset: u64,
    memory_map: &'static MemoryMap,
) -> Result<(), MapToError<Size4KiB>> {
    let mut memory_mapper = unsafe { init_memory_mapper(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootLoaderFrameAllocator::new(&memory_map) };

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            memory_mapper
                .map_to(page, frame, flags, &mut frame_allocator)?
                .flush()
        };
    }

    ALLOCATOR.init();

    Ok(())
}
