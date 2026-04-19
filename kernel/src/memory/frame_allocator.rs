use core::{
    iter::{Filter, FlatMap, Map, StepBy},
    ops::{DerefMut, Range},
    slice::Iter,
};

use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB, mapper::MapToError,
    },
};

pub static FRAME_ALLOCATOR: OnceCell<Mutex<BootLoaderFrameAllocator>> = OnceCell::uninit();
pub static MEMORY_MAPPER: OnceCell<OffsetPageTable<'_>> = OnceCell::uninit();

type RegionIterator = Iter<'static, MemoryRegion>;
type UsableRegionIterator = Filter<RegionIterator, fn(&&MemoryRegion) -> bool>;
type AddrRangeIterator = Map<UsableRegionIterator, fn(&MemoryRegion) -> Range<u64>>;
type FrameAddrIterator =
    FlatMap<AddrRangeIterator, StepBy<Range<u64>>, fn(Range<u64>) -> StepBy<Range<u64>>>;
type FrameIterator = Map<FrameAddrIterator, fn(u64) -> PhysFrame>;

pub struct BootLoaderFrameAllocator {
    frame_iterator: FrameIterator,
}

impl BootLoaderFrameAllocator {
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        Self {
            frame_iterator: Self::usable_frames(memory_map),
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(memory_map: &'static MemoryMap) -> FrameIterator {
        let regions: RegionIterator = memory_map.iter();
        let usable_regions: UsableRegionIterator =
            regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges: AddrRangeIterator =
            usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frames_addr: FrameAddrIterator = addr_ranges.flat_map(|r| r.step_by(4096));
        frames_addr.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootLoaderFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.frame_iterator.next()
    }
}

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

pub fn to_virtual_address(physical_address: PhysAddr, physical_memory_offset: u64) -> VirtAddr {
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

pub fn map_pages(
    start: VirtAddr,
    size: usize,
    flags: PageTableFlags,
    memory_mapper: &mut OffsetPageTable<'_>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let end = start + size as u64 - 1u64;
        let start_page = Page::containing_address(start);
        let end_page = Page::containing_address(end);
        Page::range_inclusive(start_page, end_page)
    };

    let mut frame_allocator = FRAME_ALLOCATOR
        .get()
        .expect("Frame allocator not initialized")
        .lock();

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            memory_mapper
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | flags,
                    frame_allocator.deref_mut(),
                )?
                .flush()
        };
    }

    Ok(())
}
