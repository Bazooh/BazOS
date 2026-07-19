use x86_64::instructions::port::Port;

use super::{HardwareInterrupt, PICS};
use crate::{r#async::SCANCODE_STREAMER, interrupts::idt::ExceptionStackFrame};

pub extern "C" fn keyboard_handler(_stack_frame: &ExceptionStackFrame) {
    let mut port = Port::new(0x60);
    SCANCODE_STREAMER
        .try_get()
        .expect("Keyboard streamer uninitialized")
        .push(unsafe { port.read() });

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(HardwareInterrupt::Keyboard as u8);
    }
}
