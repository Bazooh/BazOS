#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
extern crate alloc;

pub mod fs;
pub mod interrupts;
pub mod io_error;
pub mod keyboard;
pub mod opened_resource;
pub mod path;
pub mod qemu;
pub mod syscall;
pub mod syscall_errors;
pub mod tests;
pub mod utils;
