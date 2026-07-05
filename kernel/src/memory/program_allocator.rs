use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{NonNull, null_mut, slice_from_raw_parts_mut},
};

use crate::memory::memory_mapper::PageMapper;
use crate::memory::{
    PAGE_SIZE,
    binary_allocator::BinaryAllocator,
    buddy_allocator::{BuddyAllocator, compute_max_depth},
};
use alloc::alloc::{AllocError, Allocator};
use spin::Mutex;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, PageTableFlags, Size4KiB, mapper::MapToError},
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

unsafe impl GlobalAlloc for ProgramAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = self.allocator.lock().compute_size(layout);
        self.allocator
            .lock()
            .alloc(size)
            .unwrap_or_else(|| null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = self.allocator.lock().compute_size(layout);
        self.allocator.lock().dealloc(ptr, size);
    }
}

unsafe impl Allocator for &ProgramAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { self.alloc(layout) };
        if ptr.is_null() {
            return Err(AllocError);
        }
        Ok(NonNull::new(slice_from_raw_parts_mut(ptr, layout.size())).unwrap())
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { self.dealloc(ptr.as_ptr(), layout) };
    }
}

pub fn init_program_allocator(page_mapper: &mut PageMapper) -> Result<(), MapToError<Size4KiB>> {
    page_mapper.map(
        VirtAddr::new(PROGRAM_START),
        PROGRAM_SIZE,
        PageTableFlags::WRITABLE,
    )?;

    PROGRAM_ALLOCATOR.init();

    Ok(())
}
