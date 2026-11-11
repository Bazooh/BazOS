#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
extern crate alloc;

use crate::syscalls::exit::exit;
use crate::syscalls::out::writer::Writer;
pub use common;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

mod memory;
pub mod syscalls;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    static PANICKING: AtomicBool = AtomicBool::new(false);
    if !PANICKING.swap(true, Ordering::Relaxed) {
        let _ = writeln!(Writer::stderr(), "{}", info);
    }
    exit(1);
}
