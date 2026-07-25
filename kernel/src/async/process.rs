use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::fmt::{Debug, Formatter};

use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::thread::{STACK_SIZE, USER_STACK_START};
use crate::r#async::thread::{Thread, ThreadId};
use crate::memory::allocator::Allocator;
use crate::memory::memory_mapper::{KernelMapper, MemoryMapper, MemoryTranslator};
use crate::memory::{PAGE_SIZE, PROGRAM_ALLOCATOR, PROGRAM_HEAP_START};
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::DerefMut;
use core::ptr::copy_nonoverlapping;
use x86_64::VirtAddr;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB,
};

pub struct Process {
    pid: u64,
    parent_pid: u64,
    name: String,
    page_table: OffsetPageTable<'static>,
    last_thread_id: u64,

    memory_regions: Vec<PageRange>,
    threads_stack: BTreeMap<ThreadId, PageRange>,
}

impl Debug for Process {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Process")
            .field("pid", &self.pid)
            .field("parent_pid", &self.parent_pid)
            .field("name", &self.name)
            .finish()
    }
}

impl Process {
    pub fn user(name: String, parent_pid: u64, entry_point: VirtAddr, args: &[&str]) -> Self {
        let mut process = Process {
            pid: Scheduler::next_pid(),
            parent_pid,
            name,
            page_table: Self::create_user_table(),
            last_thread_id: 0,

            memory_regions: Vec::new(),
            threads_stack: BTreeMap::new(),
        };

        let main_thread = process.create_main_thread(entry_point, args);
        Scheduler::lock().add_thread(main_thread);

        process
    }

    pub fn kernel(name: String, parent_pid: u64, entry_point: VirtAddr, args: &[&str]) -> Self {
        let pid = Scheduler::next_pid();

        const KERNEL_STACK_SIZE: usize = STACK_SIZE as usize;
        let stack =
            Box::leak(unsafe { Box::<[u8; KERNEL_STACK_SIZE]>::new_uninit().assume_init() });

        let kernel_stack =
            Box::leak(unsafe { Box::<[u8; KERNEL_STACK_SIZE]>::new_uninit().assume_init() });

        let main_thread = Thread::kernel(
            ThreadId { pid, thread_id: 0 },
            VirtAddr::from_ptr(stack) + STACK_SIZE,
            VirtAddr::from_ptr(kernel_stack) + STACK_SIZE,
            entry_point,
            MemoryMapper::kernel().addr(),
            args,
        );

        Scheduler::lock().add_thread(main_thread);

        Process {
            pid,
            parent_pid,
            name,
            page_table: unsafe {
                MemoryMapper::page_table_from_addr(MemoryMapper::kernel().addr())
            },
            last_thread_id: 0,

            memory_regions: Vec::new(),
            threads_stack: BTreeMap::new(),
        }
    }

    fn after_fork(
        name: String,
        parent_pid: u64,
        forked_process: &Process,
        forked_thread_id: ThreadId,
        stack_pointer: VirtAddr,
    ) -> Self {
        let mut process = Process {
            pid: Scheduler::next_pid(),
            parent_pid,
            name,
            page_table: Self::create_user_table(),
            last_thread_id: 0,

            memory_regions: Vec::new(),
            threads_stack: BTreeMap::new(),
        };

        let main_thread =
            process.create_main_thread_after_fork(forked_process, forked_thread_id, stack_pointer);
        Scheduler::lock().add_thread(main_thread);

        process
    }

    #[must_use]
    fn create_main_thread(&mut self, entry_point: VirtAddr, args: &[&str]) -> Thread {
        let thread_id = ThreadId {
            pid: self.pid,
            thread_id: 0,
        };

        let stack_start =
            Page::from_start_address(USER_STACK_START).expect("User stack not aligned");
        unsafe {
            self.user_stack_mmap(
                PageRange {
                    start: stack_start,
                    end: stack_start + (STACK_SIZE / PAGE_SIZE),
                },
                thread_id,
            )
        }
        .expect("mapping failed");

        const KERNEL_STACK_SIZE: usize = STACK_SIZE as usize;
        let stack =
            Box::leak(unsafe { Box::<[u8; KERNEL_STACK_SIZE]>::new_uninit().assume_init() });

        let page_table_addr = MemoryMapper::kernel()
            .to_phys(VirtAddr::from_ptr(self.page_table.level_4_table()))
            .expect("page table translation");

        Thread::user(
            thread_id,
            stack_start.start_address() + STACK_SIZE,
            VirtAddr::from_ptr(stack) + KERNEL_STACK_SIZE as u64,
            entry_point,
            page_table_addr,
            args,
        )
    }

    #[must_use]
    fn create_main_thread_after_fork(
        &mut self,
        forked_process: &Process,
        forked_thread_id: ThreadId,
        stack_pointer: VirtAddr,
    ) -> Thread {
        fn copy_range(process: &mut Process, other: &Process, range: &PageRange) {
            for src_page in *range {
                let src_frame = other
                    .page_table
                    .translate_page(src_page)
                    .expect("page translation");
                let dst_frame = process
                    .page_table
                    .translate_page(src_page)
                    .expect("page translation");
                let src = MemoryMapper::to_page(src_frame);
                let dst = MemoryMapper::to_page(dst_frame);
                unsafe {
                    copy_nonoverlapping(
                        src.start_address().as_ptr::<u8>(),
                        dst.start_address().as_mut_ptr(),
                        PAGE_SIZE as usize,
                    );
                }
            }
        }

        fn map_and_copy_range(process: &mut Process, other: &Process, range: &PageRange) {
            unsafe { process.user_mmap(*range) }.expect("mapping failed");
            copy_range(process, other, range);
        }

        fn map_and_copy_stack_range(
            process: &mut Process,
            other: &Process,
            range: &PageRange,
            thread_id: ThreadId,
        ) {
            unsafe { process.user_stack_mmap(*range, thread_id) }.expect("mapping failed");
            copy_range(process, other, range);
        }

        let pid = self.pid;
        let stack_end = {
            for range in &forked_process.memory_regions {
                map_and_copy_range(self, forked_process, range);
            }
            // TODO: Copy the thread to the main thread slot. WARNING: THIS WON'T WORK WHEN ADDING THREADS
            let range = forked_process
                .threads_stack
                .get(&forked_thread_id)
                .expect("thread does not have a stack in process");
            map_and_copy_stack_range(self, forked_process, range, ThreadId { pid, thread_id: 0 });

            range.start.start_address() + STACK_SIZE
        };
        let thread_id = ThreadId { pid, thread_id: 0 };

        let page_table_addr = MemoryMapper::kernel()
            .to_phys(VirtAddr::from_ptr(self.page_table.level_4_table()))
            .expect("page table translation");

        const KERNEL_STACK_SIZE: usize = STACK_SIZE as usize;
        let stack =
            Box::leak(unsafe { Box::<[u8; KERNEL_STACK_SIZE]>::new_uninit().assume_init() });

        // WARNING: Same here stack_pointer will probably be wrong with multiple threads
        Thread::after_fork(
            thread_id,
            stack_end,
            VirtAddr::from_ptr(stack) + KERNEL_STACK_SIZE as u64,
            stack_pointer,
            page_table_addr,
            pid,
        )
    }

    fn create_user_table() -> OffsetPageTable<'static> {
        let page = (&PROGRAM_ALLOCATOR)
            .allocate_frames_zeroed(1)
            .expect("alloc failed")
            .start;
        let level_4_table = unsafe { &mut *page.start_address().as_mut_ptr::<PageTable>() };

        // TODO: If a lvl4 entry is after added to MEMORY_MAPPER it won't be added here
        for i in 0..256 {
            level_4_table[i] = MemoryMapper::kernel().copy_lvl4_entry(i);
        }

        // TODO: If we don't free the Box when the process finishes we leak
        unsafe { OffsetPageTable::new(level_4_table, MemoryMapper::phys_offset()) }
    }

    #[must_use]
    pub fn fork(&mut self, forked_thread_id: ThreadId, stack_pointer: VirtAddr) -> Self {
        Process::after_fork(
            self.name.clone(),
            self.parent_pid,
            self,
            forked_thread_id,
            stack_pointer,
        )
    }

    /// Safety: the desired Virtual Address range should not already be mapped
    pub unsafe fn user_mmap(&mut self, range: PageRange) -> Result<(), MapToError<Size4KiB>> {
        unsafe { self.mmap(range, &PROGRAM_ALLOCATOR, true) }?;
        self.memory_regions.push(range);
        Ok(())
    }

    /// Safety: the desired Virtual Address range should not already be mapped
    pub unsafe fn user_stack_mmap(
        &mut self,
        range: PageRange,
        thread_id: ThreadId,
    ) -> Result<(), MapToError<Size4KiB>> {
        unsafe { self.mmap(range, &PROGRAM_ALLOCATOR, true) }?;
        self.threads_stack.insert(thread_id, range);
        Ok(())
    }

    /// Safety: the desired Virtual Address range should not already be mapped
    unsafe fn mmap(
        &mut self,
        range: PageRange,
        allocator: impl Allocator,
        user: bool,
    ) -> Result<(), MapToError<Size4KiB>> {
        let pages_kernel_space = allocator
            .allocate_frames(range.len())
            .expect("memory allocation failed");

        let flags = if user {
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
        } else {
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        };

        for (page, page_kernel_space) in range.zip(pages_kernel_space) {
            let frame = MemoryMapper::kernel()
                .to_frame(page_kernel_space)
                .expect("invalid memory mapping address");

            unsafe {
                self.page_table.map_to(
                    page,
                    frame,
                    flags,
                    allocator.lock_frame_allocator().deref_mut(),
                )
            }?
            .flush();
        }

        Ok(())
    }

    pub fn find_free_mem_region(&self, n_pages: u64) -> PageRange {
        let mut intervals = self.memory_regions.to_vec();
        intervals.sort_by_key(|i| i.start);

        let mut candidate =
            Page::<Size4KiB>::from_start_address(VirtAddr::new(PROGRAM_HEAP_START)).unwrap();

        for interval in intervals {
            if candidate + n_pages <= interval.start {
                return PageRange {
                    start: candidate,
                    end: candidate + n_pages,
                };
            }

            if candidate < interval.end {
                // Move after the overlapping interval
                candidate = interval.end;
            }
        }

        PageRange {
            start: candidate,
            end: candidate + n_pages,
        }
    }

    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn page_table(&self) -> &OffsetPageTable<'static> {
        &self.page_table
    }
}
