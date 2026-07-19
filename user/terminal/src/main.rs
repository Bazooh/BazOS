#![no_std]
#![no_main]

use main_macro::main;
use std::exec::exec;
use std::println;

#[main]
fn main() {
    println!("Hello from terminal");
    exec("commands_echo", &["It", "works"]).unwrap();
}
