#![no_std]
#![no_main]

use core::arch::asm;
use std::fork::fork;
use std::{println, serial_println};

#[unsafe(no_mangle)]
fn _start() {
    println!("Hello, world!");
    fork();
    match fork() {
        None => println!("Hello from same process"),
        Some(pid) => println!("Hello from pid {:?}", pid),
    }
    println!("End");
}
