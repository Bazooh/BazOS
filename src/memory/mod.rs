use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
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
pub unsafe fn init(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset))
    }
}
