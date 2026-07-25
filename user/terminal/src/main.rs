#![no_std]
#![no_main]

use main_macro::main;
use std::println;
use std::syscalls::exec::exec;

#[main]
fn main() {
    println!("Hello from terminal");
    exec("commands_echo", &["It", "works"]).unwrap();
}
