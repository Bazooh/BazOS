#![no_std]
#![no_main]

use main_macro::main;
use std::println;

#[main]
fn main(args: &[&str]) {
    // serial_println!("Here");
    println!("ECHO ... {}, {}", args[0], args[1]);
}
