#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(std::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use BazOS::init;
use alloc::{boxed::Box, vec::Vec};
use bootloader::{BootInfo, entry_point};
use std::hlt_loop;

entry_point!(main);

pub fn main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..16384 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
