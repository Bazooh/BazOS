use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{NonNull, null_mut, slice_from_raw_parts_mut},
};

use alloc::alloc::{AllocError, Allocator};
use spin::Mutex;
use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, OffsetPageTable, PageTableFlags, PhysFrame, Size4KiB, Translate,
        mapper::MapToError,
    },
};

use crate::memory::{
    MEMORY_MAPPER, PAGE_SIZE,
    binary_allocator::BinaryAllocator,
    buddy_allocator::{BuddyAllocator, compute_max_depth},
    frame_allocator::map_pages,
};

pub const PROGRAM_START: usize = 0xffff_c444_0000_0000;
pub const PROGRAM_SIZE: usize = 1024 * 1024; // 1 GiB

const USER_PROGRAM_MAX_DEPTH: usize = compute_max_depth(PROGRAM_SIZE, PAGE_SIZE);

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
        match self.allocator.lock().alloc(size) {
            Some(ptr) => ptr,
            None => null_mut(),
        }
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

pub fn init_program_allocator(
    memory_mapper: &mut OffsetPageTable<'_>,
) -> Result<(), MapToError<Size4KiB>> {
    map_pages(
        VirtAddr::new(PROGRAM_START as u64),
        PROGRAM_SIZE,
        PageTableFlags::WRITABLE,
        memory_mapper,
    )?;

    PROGRAM_ALLOCATOR.init();

    Ok(())
}

unsafe impl FrameAllocator<Size4KiB> for BuddyAllocator<USER_PROGRAM_MAX_DEPTH> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let ptr = self.alloc(PAGE_SIZE)?;
        PhysFrame::from_start_address(
            MEMORY_MAPPER
                .get()
                .expect("Memory mapper not initialized")
                .translate_addr(VirtAddr::from_ptr(ptr))?,
        )
        .ok()
    }
}
