#![no_std]
#![no_main]

use std::fork::fork;
use std::println;

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
