#![no_std]
#![no_main]
extern crate alloc;

use main_macro::main;
use std::common::opened_resource::OpenedResource;
use std::common::utils::print_block;
use std::println;
use std::syscalls::fs::open::open;

#[main]
fn main() {
    let mut file = open("hello_world").expect("open file failed");
    let content = file
        .read_all()
        .expect("read_all failed")
        .expect("file is empty");

    let to_print = print_block(content[..10].as_ref());
    println!("{:?}", to_print);
}
