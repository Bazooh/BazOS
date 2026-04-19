#![no_std]
#![no_main]

use std::{println, serial_println};

#[unsafe(no_mangle)]
fn _start() {
    println!("Hello, world!");
}
