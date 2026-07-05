use alloc::collections::BTreeMap;
use core::fmt::{Debug, Formatter};

use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::thread::{STACK_SIZE, USER_STACK_START};
use crate::r#async::thread::{Thread, ThreadId};
use crate::memory::{MEMORY_MAPPER, PAGE_SIZE, PROGRAM_ALLOCATOR};
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::{Allocator, GlobalAlloc, Layout};
use core::ops::DerefMut;
use core::ptr::copy_nonoverlapping;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use std::serial_println;
use x86_64::VirtAddr;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB, Translate,
};

pub struct ProcessWithoutMainThread {
    pid: u64,
    parent_pid: u64,
    name: String,
    page_table: OffsetPageTable<'static>,
    entry_point: VirtAddr,
}

impl ProcessWithoutMainThread {
    fn user(pid: u64, parent_pid: u64, name: String, entry_point: VirtAddr) -> Self {
        let page_table = Self::create_user_table();

        ProcessWithoutMainThread {
            pid,
            parent_pid,
            name,
            entry_point,
            page_table,
        }
    }

    pub fn with_main_thread(self) -> Process {
        let entry_point = self.entry_point;
        let mut process = self.to_process();
        Self::create_main_thread(&mut process, entry_point);
        process
    }

    pub fn with_main_thread_after_fork(
        self,
        forked_thread_id: ThreadId,
        stack_pointer: VirtAddr,
    ) -> Process {
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
                let src = MEMORY_MAPPER.to_page(src_frame);
                let dst = MEMORY_MAPPER.to_page(dst_frame);
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
            unsafe { process.user_mmap(*range) };
            copy_range(process, other, range);
        }

        fn map_and_copy_stack_range(
            process: &mut Process,
            other: &Process,
            range: &PageRange,
            thread_id: ThreadId,
        ) {
            unsafe { process.user_stack_mmap(*range, thread_id) };
            copy_range(process, other, range);
        }

        let mut process = self.to_process();

        {
            let scheduler = Scheduler::lock();
            let forked_process = scheduler
                .get_process(forked_thread_id.pid)
                .expect("process does not exists");

            for range in &forked_process.memory_regions {
                map_and_copy_range(&mut process, forked_process, range);
            }
            let pid = process.pid;
            // TODO: Copy the thread to the main thread slot. WARNING: THIS WON'T WORK WHEN ADDING THREADS
            map_and_copy_stack_range(
                &mut process,
                forked_process,
                forked_process
                    .threads_stack
                    .get(&forked_thread_id)
                    .expect("thread does not have a stack in process"),
                ThreadId { pid, thread_id: 0 },
            );
        }
        // WARNING: Same here stack_pointer will probably be wrong with multiple threads
        let pid = process.pid;
        Self::create_main_thread_after_fork(&mut process, stack_pointer, pid);

        process
    }

    fn to_process(self) -> Process {
        Process {
            pid: self.pid,
            parent_pid: self.parent_pid,
            name: self.name,
            page_table: self.page_table,
            last_thread_id: 0,
            memory_regions: Vec::new(),
            threads_stack: BTreeMap::new(),
        }
    }

    fn create_main_thread(process: &mut Process, entry_point: VirtAddr) {
        let thread_id = ThreadId {
            pid: process.pid,
            thread_id: 0,
        };

        let start = Page::from_start_address(USER_STACK_START).expect("User stack not aligned");
        unsafe {
            process.user_stack_mmap(
                PageRange {
                    start,
                    end: start + (STACK_SIZE / PAGE_SIZE),
                },
                thread_id,
            )
        };

        let page_table_addr = MEMORY_MAPPER
            .translate_addr(VirtAddr::from_ptr(process.page_table.level_4_table()))
            .expect("page table translation");

        let thread = Thread::new(
            thread_id,
            USER_STACK_START + STACK_SIZE,
            entry_point,
            page_table_addr,
        );
        Scheduler::lock().add_thread(thread);
    }

    fn create_main_thread_after_fork(process: &mut Process, stack_pointer: VirtAddr, rax: u64) {
        let thread_id = ThreadId {
            pid: process.pid,
            thread_id: 0,
        };

        let page_table_addr = MEMORY_MAPPER
            .translate_addr(VirtAddr::from_ptr(process.page_table.level_4_table()))
            .expect("page table translation");

        let thread = Thread::after_fork(thread_id, stack_pointer, page_table_addr, rax);
        serial_println!("After fork: {:?}", thread);
        Scheduler::lock().add_thread(thread);
        serial_println!("After fork: {:?}", process);
    }

    fn create_user_table() -> OffsetPageTable<'static> {
        let frame = unsafe {
            PROGRAM_ALLOCATOR.alloc_zeroed(
                Layout::from_size_align(PAGE_SIZE as usize, PAGE_SIZE as usize).unwrap(),
            )
        };
        let level_4_table = unsafe { &mut *(frame as *mut PageTable) };
        serial_println!("Created user table at {:p}", level_4_table);

        // TODO: If a lvl4 entry is after added to MEMORY_MAPPER it won't be added here
        for i in 0..256 {
            level_4_table[i] = MEMORY_MAPPER.copy_lvl4_entry(i);
        }

        // TODO: If we don't free the Box when the process finishes we leak
        unsafe { OffsetPageTable::new(level_4_table, MEMORY_MAPPER.phys_offset()) }
    }
}

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
    pub fn user(name: String, parent_pid: u64, entry_point: VirtAddr) -> ProcessWithoutMainThread {
        ProcessWithoutMainThread::user(Scheduler::lock().next_pid(), parent_pid, name, entry_point)
    }

    pub fn fork(&mut self, entry_point: VirtAddr) -> ProcessWithoutMainThread {
        static X: AtomicU64 = AtomicU64::new(2);
        let pid = X.fetch_add(1, Ordering::Relaxed); // TODO Generate a pid (be careful of deadlock)

        ProcessWithoutMainThread::user(pid, self.parent_pid, self.name.clone(), entry_point)
    }

    // /// Safety: the desired Virtual Address range should not already be mapped
    // pub unsafe fn kernel_mmap(&mut self, n_pages: usize, start_page: Page) {
    //     self.mmap(n_pages, start_page, &KERNEL_ALLOCATOR, FRAME_A)
    // }

    /// Safety: the desired Virtual Address range should not already be mapped
    pub unsafe fn user_mmap(&mut self, range: PageRange) {
        unsafe {
            self.mmap(
                range,
                &PROGRAM_ALLOCATOR,
                PROGRAM_ALLOCATOR.frame_allocator(),
            );
            self.memory_regions.push(range);
        }
    }

    /// Safety: the desired Virtual Address range should not already be mapped
    pub unsafe fn user_stack_mmap(&mut self, range: PageRange, thread_id: ThreadId) {
        unsafe {
            self.mmap(
                range,
                &PROGRAM_ALLOCATOR,
                PROGRAM_ALLOCATOR.frame_allocator(),
            );
            self.threads_stack.insert(thread_id, range);
        }
    }

    /// Safety: the desired Virtual Address range should not already be mapped
    unsafe fn mmap(
        &mut self,
        range: PageRange,
        allocator: impl Allocator,
        frame_allocator: &Mutex<impl FrameAllocator<Size4KiB>>,
    ) {
        unsafe {
            let page_kernel_space = Page::<Size4KiB>::from_start_address(VirtAddr::from_ptr(
                allocator
                    .allocate(
                        Layout::from_size_align(
                            (range.len() * PAGE_SIZE) as usize,
                            PAGE_SIZE as usize,
                        )
                        .expect("layout not aligned"),
                    )
                    .expect("memory allocation failed")
                    .as_ptr(),
            ))
            .expect("allocation did not start at the start of a page");

            for (i, page) in range.enumerate() {
                let frame = MEMORY_MAPPER
                    .translate_page(page_kernel_space + i as u64)
                    .expect("invalid memory mapping address");

                serial_println!("Mapping {:?} to {:?}", page, frame);

                self.page_table
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::USER_ACCESSIBLE,
                        frame_allocator.lock().deref_mut(),
                    )
                    .unwrap()
                    .flush();
            }
        }
    }

    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn page_table(&self) -> &OffsetPageTable<'static> {
        &self.page_table
    }
}
