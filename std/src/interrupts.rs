#[inline]
pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

#[inline]
pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

#[inline]
pub fn without_interrupts<F: FnOnce() -> R, R>(f: F) -> R {
    x86_64::instructions::interrupts::without_interrupts(f)
}
