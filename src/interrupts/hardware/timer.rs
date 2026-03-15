use crate::interrupts::{
    hardware::{HardwareInterrupt, PICS},
    idt::ExceptionStackFrame,
};

pub extern "C" fn timer_handler(_stack_frame: &ExceptionStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(HardwareInterrupt::Timer as u8);
    }
}
