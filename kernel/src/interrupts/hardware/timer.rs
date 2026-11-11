use super::{HardwareInterrupt, PICS};
use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::gdt::TSS;
use crate::interrupts::idt::ExceptionStackFrame;
use x86_64::{PhysAddr, VirtAddr};

/// Returns whether a context switch should happen. The picked thread is stored in the
/// worker rather than returned, so that this handler never allocates.
pub extern "C" fn timer_handler(_stack_frame: &ExceptionStackFrame) -> bool {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(HardwareInterrupt::Timer as u8);
    }

    let Some(thread) = Scheduler::lock().pick_thread() else {
        return false;
    };
    Worker::current_mut().set_next_thread(thread);
    true
}

pub extern "C" fn context_switch(stack_pointer: VirtAddr) -> (VirtAddr, PhysAddr) {
    let thread = Worker::current_mut()
        .take_next_thread()
        .expect("Context switch without a next thread");
    let result = (thread.stack_pointer(), thread.page_table_address());
    unsafe { TSS.set_privilege_stack(0, thread.kernel_stack_pointer()) };

    let old_thread = Worker::current_mut().swap_thread(Some(thread));

    if let Some(thread) = old_thread {
        thread.set_stack_pointer(stack_pointer);
    }

    result
}
