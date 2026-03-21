#![no_std]
#![no_main]
#![feature(custom_test_frameworks, int_lowest_highest_one, unboxed_closures)]
#![test_runner(BazOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use BazOS::{ALLOCATOR, hlt_loop, init};
use alloc::vec::Vec;
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

    large_vec();

    println!("It did not crash!");
    hlt_loop();
}

fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
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
