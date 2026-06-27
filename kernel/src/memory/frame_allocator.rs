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

pub fn to_virtual_address(physical_address: PhysAddr, physical_memory_offset: u64) -> VirtAddr {
    VirtAddr::new(physical_address.as_u64() + physical_memory_offset)
}
