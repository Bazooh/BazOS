#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(BazOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use BazOS::{init, memory::heap::HEAP_SIZE};
use alloc::{boxed::Box, vec::Vec};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(main);

pub fn main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    BazOS::panic_handler_for_tests(info)
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
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

// #[test_case]
// fn many_boxes_long_lived() {
//     let long_lived = Box::new(1); // new
//     for i in 0..HEAP_SIZE {
//         let x = Box::new(i);
//         assert_eq!(*x, i);
//     }
//     assert_eq!(*long_lived, 1); // new
// }
