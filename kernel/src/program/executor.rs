use core::ptr::{copy_nonoverlapping, write_bytes};

use crate::memory::memory_mapper::MemoryMapper;
use crate::{
    r#async::process::Process,
    fs::{elf::header::ElfHeader, file::File},
    memory::PAGE_SIZE,
    utils::interval::{Interval, merge_intervals},
};
use alloc::{string::String, vec::Vec};
use x86_64::structures::paging::page::PageRange;
use x86_64::{
    VirtAddr,
    structures::paging::{Page, Translate},
};

pub struct ProgramExecutor {}

impl ProgramExecutor {
    pub fn execute(file: impl File, args: &[&str], parent_pid: u64) -> Process {
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
                virt_addr.as_u64(),
                number_pages * PAGE_SIZE,
            ));
        }
        let intervals = merge_intervals(intervals);

        let mut process = Process::user(
            String::from(file.name()),
            parent_pid,
            elf_header.entry_point(),
            args,
        );

        for interval in intervals {
            let start = Page::from_start_address(VirtAddr::new(interval.start()))
                .expect("Invalid start address");
            unsafe {
                process.user_mmap(PageRange {
                    start,
                    end: start + interval.size() / PAGE_SIZE,
                })
            }
            .expect("mapped memory failed");
        }

        for program_header in elf_header.program_headers() {
            if !program_header.should_load() {
                continue;
            }

            let dst_ptr = MemoryMapper::to_virt(
                process
                    .page_table()
                    .translate_addr(program_header.virt_addr())
                    .expect("failed to translate"),
            )
            .as_mut_ptr();
            let mem_size = program_header.mem_size();
            let file_size = program_header.file_size();

            unsafe {
                let file_ptr = content.as_ptr().add(program_header.offset() as usize);
                copy_nonoverlapping(file_ptr, dst_ptr, file_size as usize);
                write_bytes(
                    dst_ptr.add(file_size as usize),
                    0,
                    (mem_size - file_size) as usize,
                );
            }
        }

        process
    }
}
