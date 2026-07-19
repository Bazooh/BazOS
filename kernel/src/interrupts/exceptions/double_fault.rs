use crate::interrupts::idt::ExceptionStackFrame;

pub extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n  with error code {}\n{:#?}",
        error_code, stack_frame,
    );
}
