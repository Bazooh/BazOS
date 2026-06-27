use alloc::collections::LinkedList;
use core::fmt::{Debug, Formatter};

use crate::r#async::{
    scheduler::Scheduler,
    thread::{Thread, ThreadId},
};
use alloc::string::String;
use x86_64::VirtAddr;
use x86_64::structures::paging::OffsetPageTable;

pub struct Process {
    pid: u64,
    parent_pid: u64,
    name: String,
    page_table: OffsetPageTable<'static>,
    entry_point: VirtAddr,
    threads: LinkedList<Thread>,
    last_thread_id: u64,
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
    pub fn new(
        name: String,
        parent_pid: u64,
        entry_point: VirtAddr,
        page_table: OffsetPageTable<'static>,
    ) -> Process {
        let mut process = Process {
            pid: Scheduler::get().next_pid(),
            parent_pid,
            name,
            page_table,
            entry_point,
            threads: LinkedList::new(),
            last_thread_id: 0,
        };
        process.create_main_thread();

        process
    }

    fn create_main_thread(&mut self) {
        assert_eq!(self.last_thread_id, 0);
        let thread = Thread::new(
            ThreadId {
                pid: self.pid,
                thread_id: self.last_thread_id,
            },
            &mut self.page_table,
            self.entry_point,
        );
        self.threads.push_back(thread);
    }

    pub fn add_thread(&mut self, thread: Thread) {
        self.threads.push_back(thread);
    }

    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn threads(&self) -> impl Iterator<Item = &Thread> {
        self.threads.iter()
    }
}
