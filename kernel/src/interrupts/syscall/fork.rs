use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::r#async::thread::ThreadId;
use crate::interrupts::idt::ExceptionStackFrame;
use crate::memory::MEMORY_MAPPER;
use core::arch::asm;
use std::serial_println;
use x86_64::structures::paging::Translate;
use x86_64::{PhysAddr, VirtAddr};

pub fn fork_handler(frame: &ExceptionStackFrame) -> isize {
    let current_thread = unsafe { Worker::current_thread() };
    let thread_id = current_thread.id();

    let process = Scheduler::lock()
        .get_process_mut(thread_id.pid)
        .expect("Could not get process from pid")
        .fork(frame.instruction_pointer);

    serial_println!("switch to addr: {:?}", frame);
    serial_println!("frame pointer: {:p}", frame);

    // Drop the Scheduler::lock() before executing this
    let process = process.with_main_thread_after_fork(thread_id, VirtAddr::from_ptr(frame));
    Scheduler::lock().add_process(process);

    0
}
