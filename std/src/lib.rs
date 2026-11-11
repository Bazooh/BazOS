#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
extern crate alloc;

use crate::syscalls::exit::exit;
pub use common;
use core::panic::PanicInfo;

mod memory;
pub mod syscalls;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    exit(1);
}
