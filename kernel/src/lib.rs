#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks, unboxed_closures)]
#![test_runner(std::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use common::hlt_loop;

use bootloader::BootInfo;
#[cfg(test)]
use bootloader::entry_point;

use crate::r#async::init_async;
use crate::gdt::init_gdt;
use crate::memory::init_memory;

pub mod r#async;
mod cpu;
pub mod fs;
mod gdt;
mod interrupts;
pub mod io;
pub mod memory;
pub mod out;
pub mod program;
mod utils;

#[cfg(test)]
entry_point!(main);

#[cfg(test)]
pub fn main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}

pub fn init(boot_info: &'static BootInfo) {
    interrupts::disable();
    init_gdt();
    interrupts::init_idt();
    init_memory(boot_info.physical_memory_offset, &boot_info.memory_map);
    init_async();
    io::device::init();
}
