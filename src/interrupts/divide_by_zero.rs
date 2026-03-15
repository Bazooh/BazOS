use core::arch::asm;

use crate::{interrupts::idt::ExceptionStackFrame, println};

pub extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
    loop {}
}

#[test_case]
fn test_divide_by_zero() {
    unsafe {
        asm!(
            "xor rdx, rdx", // set high part of dividend to 0
            "mov rax, 1",   // set low part of dividend to 1
            "xor rcx, rcx", // set divisor to 0
            "div rcx",      // divide by zero -> CPU exception
            options(noreturn)
        );
    }
}
