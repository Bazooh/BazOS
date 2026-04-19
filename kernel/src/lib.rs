#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(
    allocator_api,
    custom_test_frameworks,
    int_lowest_highest_one,
    unboxed_closures
)]
#![test_runner(std::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use core::str::from_utf8;
#[cfg(test)]
use std::hlt_loop;
use std::serial_println;

use bootloader::BootInfo;
#[cfg(test)]
use bootloader::entry_point;

use crate::r#async::executor::Executor;
use crate::r#async::init_async;
use crate::gdt::init_gdt;
use crate::memory::init_memory;

pub mod r#async;
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
    serial_println!("Tst");
    test_main();
    hlt_loop();
}

pub fn init(boot_info: &'static BootInfo) {
    interrupts::disable();
    init_gdt();
    interrupts::init_idt();
    init_memory(boot_info.physical_memory_offset, &boot_info.memory_map);
    init_async();
    interrupts::enable();
    io::device::init();
}

pub fn print_data(data: &[u8]) {
    let address = data.as_ptr() as u64;
    for (i, line) in data.chunks(16).enumerate() {
        serial_println!(
            "0x{}:  {:<47}  |{:<16}|",
            format!("{:#018x}", address + i as u64 * 16)[2..]
                .as_bytes()
                .rchunks(4)
                .rev()
                .map(|c| from_utf8(c).unwrap())
                .collect::<Vec<_>>()
                .join("_"),
            line.chunks(8)
                .map(|g| {
                    g.iter()
                        .map(|x| format!("{x:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join("  "),
            line.iter()
                .map(|x| match *x {
                    0x20..=0x7e => char::from(*x),
                    _ => '.',
                })
                .collect::<String>()
        );
    }
}
