use super::{HardwareInterrupt, PICS};
use crate::r#async::ThreadRef;
use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::gdt::TSS;
use crate::interrupts::idt::ExceptionStackFrame;
use alloc::boxed::Box;
use std::serial_println;
use x86_64::{PhysAddr, VirtAddr};

pub extern "C" fn timer_handler(_stack_frame: &ExceptionStackFrame) -> Option<Box<ThreadRef>> {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(HardwareInterrupt::Timer as u8);
    }

    serial_println!("timer handler");

    Scheduler::lock()
        .pick_thread()
        .map(|thread| Box::new(thread))
}

pub extern "C" fn context_switch(
    stack_pointer: VirtAddr,
    thread: Box<ThreadRef>,
) -> (VirtAddr, PhysAddr) {
    let result = (thread.stack_pointer(), thread.page_table_address());
    unsafe { TSS.set_privilege_stack(0, thread.kernel_stack_pointer()) };

    let old_thread = Worker::current_mut().swap_thread(Some(thread));

    if let Some(thread) = old_thread {
        thread.set_stack_pointer(stack_pointer);
    }

    result
}
