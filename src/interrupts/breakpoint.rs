use crate::{interrupts::idt::ExceptionStackFrame, println};

pub extern "C" fn breakpoint_handler(stack_frame: &ExceptionStackFrame) {
    println!("BREAKPOINT:\n{:#?}", stack_frame);
}
