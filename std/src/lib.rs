#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]

use core::panic::PanicInfo;

pub mod fs;
pub mod interrupts;
pub mod out;
pub mod qemu;
pub mod syscall;
pub mod tests;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupts::disable();
    serial_println!("{}\n", info);
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupts::disable();
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit(qemu::ExitCode::Failed);
}
