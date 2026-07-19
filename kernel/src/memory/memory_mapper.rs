use crate::cpu::registers::Register;
use crate::cpu::registers::cr3;
use crate::memory::bootloader_allocator::FRAME_ALLOCATOR;
use crate::memory::heap::init_heap;
use crate::memory::program_allocator::init_program_allocator;
use crate::memory::to_virtual_address;
use conquer_once::spin::OnceCell;
use core::arch::asm;
use core::ops::DerefMut;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    Translate,
};
use x86_64::{PhysAddr, VirtAddr};

static KERNEL_MAPPER: OnceCell<OffsetPageTable<'static>> = OnceCell::uninit();

pub struct MemoryMapper {
    mapper: &'static OffsetPageTable<'static>,
}

pub trait MemoryTranslator {
    fn to_phys(&self, virt: VirtAddr) -> Option<PhysAddr>;
    fn to_frame(&self, page: Page) -> Option<PhysFrame>;
}

impl MemoryTranslator for OffsetPageTable<'_> {
    fn to_phys(&self, virt: VirtAddr) -> Option<PhysAddr> {
        Translate::translate_addr(self, virt)
    }

    fn to_frame(&self, page: Page) -> Option<PhysFrame> {
        Mapper::translate_page(self, page).ok()
    }
}

pub trait KernelMapper: MemoryTranslator {
    fn addr(&self) -> PhysAddr;
    fn copy_lvl4_entry(&self, i: usize) -> PageTableEntry;
}

impl KernelMapper for OffsetPageTable<'static> {
    fn addr(&self) -> PhysAddr {
        Translate::translate_addr(self, VirtAddr::from_ptr(self.level_4_table())).unwrap()
    }

    fn copy_lvl4_entry(&self, i: usize) -> PageTableEntry {
        self.level_4_table()[i].clone()
    }
}

impl MemoryMapper {
    pub fn kernel<'a>() -> &'a impl KernelMapper {
        KERNEL_MAPPER.get().expect("Memory mapper not initialized")
    }

    pub fn current() -> impl MemoryTranslator {
        unsafe { Self::page_table_from_addr(PhysAddr::new(cr3().read())) }
    }

    fn get_kernel_mapper() -> &'static OffsetPageTable<'static> {
        KERNEL_MAPPER.get().expect("Memory mapper not initialized")
    }

    pub fn phys_offset() -> VirtAddr {
        Self::get_kernel_mapper().phys_offset()
    }

    pub fn to_virt(phys_addr: PhysAddr) -> VirtAddr {
        Self::phys_offset() + phys_addr.as_u64()
    }

    pub fn to_page(frame: PhysFrame) -> Page {
        let virt_addr = Self::to_virt(frame.start_address());
        Page::from_start_address(virt_addr).unwrap()
    }

    /// Initialize a new OffsetPageTable.
    ///
    /// This function is unsafe because the caller must guarantee that the
    /// complete physical memory is mapped to virtual memory at the passed
    /// `physical_memory_offset`. Also, this function must be only called once
    /// to avoid aliasing `&mut` references (which is undefined behavior).
    pub unsafe fn init(physical_memory_offset: u64) {
        let mut page_table = unsafe {
            let level_4_table = Self::active_level_4_table(physical_memory_offset);
            OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset))
        };

        let mut page_mapper = PageMapper {
            mapper: &mut page_table,
        };

        init_heap(&mut page_mapper);
        init_program_allocator(&mut page_mapper);

        KERNEL_MAPPER
            .try_init_once(|| page_table)
            .expect("Physical Level 4 Address already initialized");
    }

    pub unsafe fn page_table_from_addr(page_table_addr: PhysAddr) -> OffsetPageTable<'static> {
        let ptr = Self::to_virt(page_table_addr).as_mut_ptr();
        unsafe { OffsetPageTable::new(&mut *ptr, Self::phys_offset()) }
    }

    /// Returns a mutable reference to the active level 4 table.
    ///
    /// This function is unsafe because the caller must guarantee that the
    /// complete physical memory is mapped to virtual memory at the passed
    /// `physical_memory_offset`. Also, this function must be only called once
    /// to avoid aliasing `&mut` references (which is undefined behavior).
    unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
        let address = to_virtual_address(PhysAddr::new(cr3().read()), physical_memory_offset);
        unsafe { &mut *(address.as_mut_ptr()) }
    }

    pub unsafe fn switch_to(page_table_addr: PhysAddr) -> PhysAddr {
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

pub struct PageMapper<'a> {
    mapper: &'a mut OffsetPageTable<'static>,
}

impl<'a> PageMapper<'a> {
    pub fn map(
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
                    .map_to(page, frame, flags, frame_allocator.deref_mut())?
                    .flush()
            };
        }

        Ok(())
    }
}
