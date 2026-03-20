#![allow(non_snake_case)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks, int_lowest_highest_one)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::ops::Fn;
use core::panic::PanicInfo;

use bootloader::BootInfo;
#[cfg(test)]
use bootloader::entry_point;
use x86_64::instructions::port::Port;

use crate::gdt::init_gdt;
use crate::memory::Allocator;
use crate::memory::heap::init_heap;

mod gdt;
mod interrupts;
pub mod memory;
mod serial;
mod utils;
mod vga;

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[cfg(test)]
entry_point!(main);

#[cfg(test)]
pub fn main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init(boot_info: &'static BootInfo) {
    interrupts::disable();
    init_gdt();
    interrupts::init_idt();
    interrupts::enable();
    init_heap(boot_info.physical_memory_offset, &boot_info.memory_map)
        .expect("Heap initialization failed");
}

pub fn panic_handler_for_tests(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    panic_handler_for_tests(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    let mut port = Port::new(0xf4);
    unsafe {
        port.write(exit_code as u32);
    }
    loop {}
}
