use crate::memory::allocator::Allocator;
use crate::memory::binary_allocator::BinaryAllocator;
use crate::memory::memory_mapper::PageMapper;
use crate::memory::{
    PAGE_SIZE,
    buddy_allocator::{BuddyAllocator, compute_max_depth},
};
use core::ptr::write_bytes;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::Page;
use x86_64::structures::paging::page::PageRange;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, PageTableFlags, Size4KiB},
};

pub const PROGRAM_START: u64 = 0x5000_0000_0000;
pub const PROGRAM_SIZE: u64 = 1024 * 1024; // 1 MiB

const USER_PROGRAM_MAX_DEPTH: usize = compute_max_depth(PROGRAM_SIZE, PAGE_SIZE) as usize;

pub static PROGRAM_ALLOCATOR: ProgramAllocator = ProgramAllocator::new();

pub struct ProgramAllocator {
    allocator: Mutex<BuddyAllocator<USER_PROGRAM_MAX_DEPTH>>,
}

impl ProgramAllocator {
    pub const fn new() -> Self {
        ProgramAllocator {
            allocator: Mutex::new(BuddyAllocator::new(PROGRAM_SIZE, PAGE_SIZE, PROGRAM_START)),
        }
    }

    pub fn init(&self) {
        self.allocator.lock().init();
    }

    pub fn frame_allocator(&self) -> &Mutex<impl FrameAllocator<Size4KiB>> {
        &self.allocator
    }
}

impl Allocator for &ProgramAllocator {
    fn allocate_frames(&self, n_frames: u64) -> Option<PageRange> {
        self.allocator
            .lock()
            .alloc(n_frames * PAGE_SIZE)
            .map(|ptr| {
                let page = Page::from_start_address(VirtAddr::from_ptr(ptr)).unwrap();
                PageRange {
                    start: page,
                    end: page + n_frames,
                }
            })
    }

    fn allocate_frames_zeroed(&self, n_frames: u64) -> Option<PageRange> {
        let pages = self.allocate_frames(n_frames)?;
        unsafe {
            write_bytes(
                pages.start.start_address().as_mut_ptr::<u8>(),
                0,
                (n_frames * PAGE_SIZE) as usize,
            )
        };
        Some(pages)
    }

    fn deallocate_frames(&self, range: PageRange) {
        self.allocator
            .lock()
            .dealloc(range.start.start_address().as_mut_ptr(), range.len())
    }

    fn lock_frame_allocator(&self) -> MutexGuard<'_, impl FrameAllocator<Size4KiB>> {
        self.allocator.lock()
    }
}

pub fn init_program_allocator(page_mapper: &mut PageMapper) {
    page_mapper
        .map(
            VirtAddr::new(PROGRAM_START),
            PROGRAM_SIZE,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        )
        .expect("Memory mapping failed");

    PROGRAM_ALLOCATOR.init();
}
