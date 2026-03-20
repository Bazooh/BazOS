use core::{
    iter::{Filter, FlatMap, Map, StepBy},
    ops::Range,
    slice::Iter,
};

use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
};

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
