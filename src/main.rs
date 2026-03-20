#![no_std]
#![no_main]
#![feature(custom_test_frameworks, int_lowest_highest_one)]
#![test_runner(BazOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use BazOS::{ALLOCATOR, hlt_loop, init, memory::heap::HEAP_SIZE};
use alloc::{boxed::Box, vec::Vec};
use bootloader::{BootInfo, entry_point};

mod gdt;
mod interrupts;
mod memory;
mod serial;
mod utils;
mod vga;

entry_point!(main);

#[allow(unreachable_code, unused_variables)]
pub fn main(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    BazOS::exit_qemu(BazOS::QemuExitCode::Success);

    init(boot_info);

    println!("It did not crash!");
    hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // fill_screen!(Blue);
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt_loop();
}
