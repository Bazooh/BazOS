#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(BazOS::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use BazOS::init;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    BazOS::panic_handler_for_tests(info)
}
