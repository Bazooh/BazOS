#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(BazOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{arch::asm, panic::PanicInfo};

use BazOS::{hlt_loop, init};
use bootloader::{BootInfo, entry_point};
use x86_64::{
    VirtAddr,
    instructions::interrupts::int3,
    structures::paging::{Mapper, Translate},
};

mod gdt;
mod interrupts;
mod memory;
mod serial;
mod utils;
mod vga;

entry_point!(main);

#[allow(unreachable_code)]
pub fn main(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    BazOS::exit_qemu(BazOS::QemuExitCode::Success);

    init();

    let page_table = unsafe { memory::init(boot_info.physical_memory_offset) };

    println!("It did not crash!");

    hlt_loop();
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
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
