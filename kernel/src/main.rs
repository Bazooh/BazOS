#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(std::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::string::String;

use BazOS::r#async::executor::Executor;
use BazOS::r#async::process::Process;
use BazOS::r#async::scheduling::scheduler::Scheduler;
use BazOS::r#async::scheduling::worker::Worker;
use BazOS::{
    fs::{driver::DiskDriver, path::Path},
    init,
    io::disk::driver::DISK_DRIVER,
    program::executor::ProgramExecutor,
};
use bootloader::{BootInfo, entry_point};
use common::qemu;
use x86_64::VirtAddr;

entry_point!(main);

#[allow(unreachable_code)]
pub fn main(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    qemu::exit(qemu::ExitCode::Success);

    init(boot_info);

    add_process_from_file("terminal", &[]);
    add_process_from_file("hello_world", &[]);
    add_process_from_file("commands_echo", &["Hello, terminal!222", "LLLL"]);

    let kernel_process = Process::kernel(
        String::from("kernel"),
        0,
        VirtAddr::from_ptr(Executor::kernel as *const ()),
        &[],
    );
    Scheduler::lock().add_process(kernel_process);

    Worker::run();
}

fn add_process_from_file(file: &str, args: &[&str]) {
    let file = DISK_DRIVER
        .try_get()
        .unwrap()
        .open(Path::new(file))
        .unwrap();

    let process = ProgramExecutor::execute(file, args, 0);
    Scheduler::lock().add_process(process);
}
