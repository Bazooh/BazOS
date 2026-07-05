#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(std::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use std::qemu::exit;

use BazOS::r#async::scheduling::worker::Worker;
use BazOS::{
    fs::{driver::DiskDriver, path::Path},
    init,
    io::disk::driver::DISK_DRIVER,
    program::executor::ProgramExecutor,
};
use bootloader::{BootInfo, entry_point};

entry_point!(main);

#[allow(unreachable_code)]
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

    Worker::new().run();
    // Executor::kernel();
}
