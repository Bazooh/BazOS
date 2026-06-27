use core::{
    alloc::{GlobalAlloc, Layout},
    intrinsics::{copy_nonoverlapping, write_bytes},
    ops::{Add, DerefMut},
};

use crate::{
    r#async::{process::Process, scheduler::Scheduler},
    fs::{elf::header::ElfHeader, file::File},
    memory::{MEMORY_MAPPER, PAGE_SIZE, PROGRAM_ALLOCATOR},
    utils::interval::{Interval, merge_intervals},
};
use alloc::boxed::Box;
use alloc::{string::String, vec::Vec};
use core::alloc::Allocator;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate,
    },
};

pub struct ProgramExecutor {}

impl ProgramExecutor {
    fn create_table() -> OffsetPageTable<'static> {
        let level_4_table = Box::leak(Box::new_in(PageTable::new(), &PROGRAM_ALLOCATOR));

        for i in 0..256 {
            level_4_table[i] = MEMORY_MAPPER.clone_page(i);
        }

        // TODO: If we don't free the Box when the process finishes we leak
        unsafe { OffsetPageTable::new(level_4_table, MEMORY_MAPPER.phys_offset()) }
    }

    fn map(mappings: Vec<(Interval, PhysAddr)>) -> OffsetPageTable<'static> {
        let mut table = Self::create_table();
        let mut frame_allocator = PROGRAM_ALLOCATOR.frame_allocator().lock();
        for (interval, phys_addr) in mappings {
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(interval.start() as u64));
            let frame = PhysFrame::containing_address(phys_addr);

            for i in 0..interval.size() as u64 {
                unsafe {
                    table
                        .map_to(
                            page + i,
                            frame + i,
                            PageTableFlags::PRESENT
                                | PageTableFlags::WRITABLE
                                | PageTableFlags::USER_ACCESSIBLE,
                            frame_allocator.deref_mut(),
                        )
                        .expect("Mapping failed")
                        .flush();
                }
            }
        }
        table
    }

    pub fn execute(file: impl File) {
        let content = file.read();
        let elf_header = ElfHeader::parse(&content).expect("Not executable");

        let mut intervals = Vec::with_capacity(elf_header.n_program_headers());
        for program_header in elf_header.program_headers() {
            if !program_header.should_load() {
                continue;
            }

            let virt_addr = program_header.virt_addr();
            let mem_size = program_header.mem_size();
            assert!(
                program_header.align() <= PAGE_SIZE,
                "Alignment not supported"
            );

            let number_pages = mem_size.div_ceil(PAGE_SIZE);
            intervals.push(Interval::with_size(
                virt_addr.as_u64() as usize,
                number_pages * PAGE_SIZE,
            ));
        }

        let intervals = merge_intervals(intervals)
            .into_iter()
            .map(|interval| {
                let layout = Layout::from_size_align(interval.size(), PAGE_SIZE).unwrap();
                let allocated_virt_ptr = (&PROGRAM_ALLOCATOR)
                    .allocate(layout)
                    .expect("Allocation failed")
                    .as_ptr()
                    .cast::<u8>();
                let phys_addr = MEMORY_MAPPER
                    .translate_addr(VirtAddr::from_ptr(allocated_virt_ptr))
                    .expect("Translation failed");

                (interval, allocated_virt_ptr, phys_addr)
            })
            .collect::<Vec<_>>();

        for program_header in elf_header.program_headers() {
            if !program_header.should_load() {
                continue;
            }

            let virt_addr = program_header.virt_addr();
            let mem_size = program_header.mem_size();
            let file_size = program_header.file_size();

            let (interval, allocated_virt_ptr, _) = intervals
                .iter()
                .find(|(interval, _, _)| interval.contains(virt_addr.as_u64() as usize))
                .expect("No interval found");

            let offset = virt_addr.as_u64() as usize - interval.start();
            let dst_ptr = unsafe { allocated_virt_ptr.add(offset) };

            unsafe {
                let file_ptr = content.as_ptr().add(program_header.offset());
                copy_nonoverlapping(file_ptr, dst_ptr, file_size);
                write_bytes(dst_ptr.add(file_size), 0, mem_size - file_size);
            }
        }

        let page_table = Self::map(
            intervals
                .into_iter()
                .map(|(interval, _, phys_addr)| (interval, phys_addr))
                .collect(),
        );
        let entry_point = elf_header.entry_point();
        let process = Process::new(String::from(file.name()), 0, entry_point, page_table);

        Scheduler::get().add_process(process);
    }
}
