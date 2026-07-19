#![no_std]
#![no_main]

use main_macro::main;
use std::fork::fork;
use std::println;

#[main]
fn main() {
    println!("Hello, world!");
    fork();
    println!("Hello, world2!");
    match fork() {
        None => println!("Hello from same process"),
        Some(pid) => println!("Hello from pid {:?}", pid),
    }
    println!("End");
}
