#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(std::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::arch::asm;
#[cfg(test)]
use std::qemu::exit;
use std::serial_println;

use BazOS::{
    r#async::scheduler::Scheduler,
    fs::{driver::DiskDriver, elf::header::ElfHeader, file::File, path::Path},
    init,
    io::disk::driver::DISK_DRIVER,
    memory::MEMORY_MAPPER,
    print_data,
    program::executor::ProgramExecutor,
};
use alloc::vec::Vec;
use bootloader::{BootInfo, bootinfo::MemoryRegionType, entry_point};

use BazOS::r#async::executor::Executor;
use x86_64::{VirtAddr, structures::paging::Translate};

entry_point!(main);

pub fn main(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    exit(std::qemu::ExitCode::Success);

    init(boot_info);

    let file = DISK_DRIVER
        .try_get()
        .unwrap()
        .open(Path::new("hello_world"))
        .unwrap();

    ProgramExecutor::execute(file);

    Scheduler::get().run();
    // Executor::kernel();
}
