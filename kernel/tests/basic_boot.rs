#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(std::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use std::hlt_loop;

use BazOS::init;
use bootloader::{BootInfo, entry_point};

entry_point!(main);

pub fn main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}
