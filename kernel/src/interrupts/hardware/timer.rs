use super::{HardwareInterrupt, PICS};
use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::r#async::thread::Thread;
use crate::interrupts::idt::ExceptionStackFrame;
use core::ops::{Deref, DerefMut};
use core::ptr::null_mut;
use std::serial_println;

pub extern "C" fn timer_handler(_stack_frame: &ExceptionStackFrame) -> *mut Thread {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(HardwareInterrupt::Timer as u8);
    }

    serial_println!("rip: {:?}", _stack_frame);

    match Scheduler::lock().pick_thread() {
        Some(mut thread) => thread.deref_mut() as *mut Thread,
        None => null_mut(),
    }
}
