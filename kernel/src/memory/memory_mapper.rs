use crate::memory::frame_allocator::FRAME_ALLOCATOR;
use crate::memory::heap::init_heap;
use crate::memory::program_allocator::init_program_allocator;
use crate::memory::to_virtual_address;
use conquer_once::spin::OnceCell;
use core::ops::{Deref, DerefMut};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::{MapToError, TranslateError};
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    Translate,
};
use x86_64::{PhysAddr, VirtAddr};

pub static MEMORY_MAPPER: MemoryMapper = MemoryMapper::new();

pub struct MemoryMapper {
    mapper: OnceCell<OffsetPageTable<'static>>,
}

impl MemoryMapper {
    const fn new() -> Self {
        MemoryMapper {
            mapper: OnceCell::uninit(),
        }
    }

    pub(crate) fn phys_offset(&self) -> VirtAddr {
        self.get_mapper().phys_offset()
    }

    pub(crate) fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        self.get_mapper().translate_addr(addr)
    }

    pub(crate) fn translate_page(&self, page: Page) -> Result<PhysFrame, TranslateError> {
        self.get_mapper().translate_page(page)
    }

    pub(crate) fn clone_page(&self, i: usize) -> PageTableEntry {
        self.get_mapper().level_4_table()[i].clone()
    }

    fn get_mapper(&self) -> &OffsetPageTable<'static> {
        self.mapper.get().expect("Mapper not initialized")
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
}

pub struct PageMapper<'a> {
    mapper: &'a mut OffsetPageTable<'static>,
}

impl<'a> PageMapper<'a> {
    pub(crate) fn map(
        &mut self,
        start: VirtAddr,
        size: usize,
        flags: PageTableFlags,
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
