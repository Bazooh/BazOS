use crate::memory::frame_allocator::FRAME_ALLOCATOR;
use crate::memory::heap::init_heap;
use crate::memory::program_allocator::init_program_allocator;
use crate::memory::to_virtual_address;
use conquer_once::spin::OnceCell;
use core::arch::asm;
use core::ops::{Deref, DerefMut};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub static MEMORY_MAPPER: MemoryMapper = MemoryMapper::new();

pub struct MemoryMapper {
    mapper: OnceCell<OffsetPageTable<'static>>,
    level_4_table_frame: OnceCell<PhysFrame>,
}

impl MemoryMapper {
    const fn new() -> Self {
        MemoryMapper {
            mapper: OnceCell::uninit(),
            level_4_table_frame: OnceCell::uninit(),
        }
    }

    pub(crate) fn copy_lvl4_entry(&self, i: usize) -> PageTableEntry {
        self.get_mapper().level_4_table()[i].clone()
    }

    fn get_mapper(&self) -> &OffsetPageTable<'static> {
        self.mapper.get().expect("Mapper not initialized")
    }

    pub fn to_virt(&self, phys_addr: PhysAddr) -> VirtAddr {
        self.phys_offset() + phys_addr.as_u64()
    }

    pub fn to_page(&self, frame: PhysFrame) -> Page {
        Page::from_start_address(self.to_virt(frame.start_address())).unwrap()
    }

    /// Initialize a new OffsetPageTable.
    ///
    /// This function is unsafe because the caller must guarantee that the
    /// complete physical memory is mapped to virtual memory at the passed
    /// `physical_memory_offset`. Also, this function must be only called once
    /// to avoid aliasing `&mut` references (which is undefined behavior).
    pub(crate) unsafe fn init(&self, physical_memory_offset: u64) {
        let mut page_table = unsafe {
            let level_4_table = Self::active_level_4_table(physical_memory_offset);
            OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset))
        };

        let mut page_mapper = PageMapper {
            mapper: &mut page_table,
        };

        self.level_4_table_frame
            .try_init_once(|| Cr3::read().0)
            .expect("Physical Level 4 Address already initialized");

        init_heap(&mut page_mapper).expect("Heap initialization failed");
        init_program_allocator(&mut page_mapper).expect("Program allocator initialization failed");

        self.mapper
            .try_init_once(|| page_table)
            .expect("Memory mapper already initialized");
    }

    /// Returns a mutable reference to the active level 4 table.
    ///
    /// This function is unsafe because the caller must guarantee that the
    /// complete physical memory is mapped to virtual memory at the passed
    /// `physical_memory_offset`. Also, this function must be only called once
    /// to avoid aliasing `&mut` references (which is undefined behavior).
    unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
        let (level_4_table_frame, _) = Cr3::read();
        let address =
            to_virtual_address(level_4_table_frame.start_address(), physical_memory_offset);
        unsafe { &mut *(address.as_mut_ptr()) }
    }

    pub unsafe fn switch_to_kernel(&self) {
        unsafe {
            self.switch_to(
                self.level_4_table_frame
                    .get()
                    .expect("Memory mapper not initialized")
                    .start_address(),
            );
        }
    }

    pub unsafe fn switch_to(&self, page_table_addr: PhysAddr) -> PhysAddr {
        let mut old_page_table_addr: u64;
        unsafe {
            asm!(
                "mov {old_cr3}, cr3",
                "mov cr3, {cr3}",
                cr3     = in(reg) page_table_addr.as_u64(),
                old_cr3 = out(reg) old_page_table_addr,
            );
        }
        PhysAddr::new(old_page_table_addr)
    }
}

impl Deref for MemoryMapper {
    type Target = OffsetPageTable<'static>;

    fn deref(&self) -> &Self::Target {
        self.get_mapper()
    }
}

pub struct PageMapper<'a> {
    mapper: &'a mut OffsetPageTable<'static>,
}

impl<'a> PageMapper<'a> {
    pub(crate) fn map(
        &mut self,
        start: VirtAddr,
        size: u64,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<Size4KiB>> {
        let page_range = {
            let end = start + size - 1u64;
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
                self.mapper
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
}
