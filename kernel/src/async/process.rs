use core::{
    arch::{asm, naked_asm},
    fmt::{Debug, Formatter},
    mem::MaybeUninit,
    ptr::slice_from_raw_parts,
};
use std::serial_println;

use alloc::{boxed::Box, string::String};
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{OffsetPageTable, PageTable, PhysFrame, Translate},
};

use crate::{
    r#async::{scheduler::Scheduler, thread::Thread},
    memory::{MEMORY_MAPPER, PAGE_SIZE},
    print_data,
};

pub struct Process {
    pid: u64,
    parent_pid: u64,
    name: String,
    page_table_frame: PhysFrame,
    entry_point: VirtAddr,
}

impl Debug for Process {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MyStruct")
            .field("pid", &self.pid)
            .field("parent_pid", &self.parent_pid)
            .field("name", &self.name)
            .finish()
    }
}

impl Process {
    pub fn new(
        name: String,
        parent_pid: u64,
        entry_point: VirtAddr,
        page_table_frame: PhysFrame,
    ) -> Process {
        Process {
            pid: Scheduler::get().next_pid(),
            parent_pid,
            name,
            page_table_frame,
            entry_point,
        }
    }

    pub fn create_main_thread(&self) -> Thread {
        Thread::new(
            self.pid,
            self.page_table_frame.start_address(),
            self.entry_point,
        )
    }

    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn page_table_addr(&self) -> PhysAddr {
        self.page_table_frame.start_address()
    }
}
