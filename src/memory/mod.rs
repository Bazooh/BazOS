use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
};

use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
};

use crate::memory::allocator::BuddyAllocator;

mod allocator;
pub mod frame_allocator;
pub mod heap;

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

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub struct Allocator {
    allocator: Mutex<BuddyAllocator>,
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            allocator: Mutex::new(BuddyAllocator::new()),
        }
    }

    pub fn init(&self) {
        self.allocator.lock().init();
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.allocator.lock().alloc(layout) {
            Some(ptr) => ptr,
            None => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.lock().dealloc(ptr, layout);
    }
}
