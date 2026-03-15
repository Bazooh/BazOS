use crate::{interrupts::idt::ExceptionStackFrame, println};

pub extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    println!(
        "EXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame
    );
    loop {}
}
