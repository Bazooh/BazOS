use spin::MutexGuard;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::{FrameAllocator, Size4KiB};

pub trait Allocator {
    fn allocate_frames(&self, n_frames: u64) -> Option<PageRange>;

    fn allocate_frames_zeroed(&self, n_frames: u64) -> Option<PageRange>;

    fn deallocate_frames(&self, range: PageRange);

    fn lock_frame_allocator(&self) -> MutexGuard<'_, impl FrameAllocator<Size4KiB>>;
}
