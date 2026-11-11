#![no_std]
#![no_main]

use main_macro::main;
use std::common::fs::FileDescriptorIndex;
use std::common::opened_resource::BLOCK_SIZE;
use std::common::syscall::SyscallNumber::{Read, Write};
use std::common::syscall::syscall;
use std::println;
use std::syscalls::exec::exec;

#[main]
fn main() {
    println!("Hello from terminal");
    exec("commands_echo", &["It", "works"]).unwrap();
    println!("Hello from terminal");

    loop {
        let buffer = [0u8; BLOCK_SIZE];
        let read = syscall(
            Read,
            FileDescriptorIndex::STDIN.as_u64(),
            buffer.as_ptr() as u64,
            BLOCK_SIZE as u64,
            0,
        );

        if read > 0 {
            syscall(
                Write,
                FileDescriptorIndex::STDOUT.as_u64(),
                buffer.as_ptr() as u64,
                read as u64,
                0,
            );
        }
    }
}
