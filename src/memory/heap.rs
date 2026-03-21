use bootloader::bootinfo::MemoryMap;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB,
        mapper::MapToError,
    },
};

use crate::{
    ALLOCATOR,
    memory::{HEAP_SIZE, HEAP_START, frame_allocator::BootLoaderFrameAllocator},
};

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let address = to_virtual_address(level_4_table_frame.start_address(), physical_memory_offset);
    unsafe { &mut *(address.as_mut_ptr()) }
}

fn to_virtual_address(physical_address: PhysAddr, physical_memory_offset: u64) -> VirtAddr {
    VirtAddr::new(physical_address.as_u64() + physical_memory_offset)
}

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init_memory_mapper(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset))
    }
}

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
