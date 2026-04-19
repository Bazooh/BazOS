#![allow(unused_imports)]

mod idt;
pub use crate::interrupts::idt::init_idt;

mod breakpoint;
mod divide_by_zero;
mod double_fault;
mod hardware;
mod invalid_opcode;
mod page_fault;
mod syscall;

#[allow(unused)]
#[inline]
pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

#[allow(unused)]
#[inline]
pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

#[allow(unused)]
#[inline]
pub fn without_interrupts<F: FnOnce() -> R, R>(f: F) -> R {
    x86_64::instructions::interrupts::without_interrupts(f)
}
