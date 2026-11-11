#![no_std]
#![no_main]
extern crate alloc;

use main_macro::main;
use std::common::fs::FileDescriptorIndex;
use std::common::opened_resource::Resource;
use std::common::syscall_errors::replace_fd::ReplaceFdError;
use std::println;
use std::syscalls::fs::file_descriptor::FileDescriptor;
use std::syscalls::fs::open::open;

const ELF_MAGIC: &[u8] = b"\x7fELF";

#[main]
fn main() {
    test_duplicate_fd();
    test_replace_fd();
    println!("DuplicateFd / ReplaceFd: all tests passed");
}

fn test_duplicate_fd() {
    let original = open("hello_world").expect("open failed");
    let mut duplicate = original.duplicate().expect("duplicate failed");
    println!(
        "duplicate: original fd {:?} -> duplicate fd {:?}",
        original.index(),
        duplicate.index()
    );
    assert_ne!(original.index(), duplicate.index());

    // The duplicate must keep the resource alive after the original is closed.
    drop(original);
    let content = duplicate
        .read_all()
        .expect("read from duplicate failed")
        .expect("duplicate is empty");
    assert_eq!(&content[..ELF_MAGIC.len()], ELF_MAGIC);
    println!(
        "duplicate: read {} bytes after closing original",
        content.len()
    );
}

fn test_replace_fd() {
    let expected = open("hello_world")
        .expect("open hello_world failed")
        .read_all()
        .expect("read hello_world failed")
        .expect("hello_world is empty");

    let source = open("hello_world").expect("open hello_world failed");
    let mut target = open("terminal").expect("open terminal failed");

    // After this, `target`'s fd refers to hello_world instead of terminal.
    source.replace(target.index()).expect("replace failed");
    let content = target
        .read_all()
        .expect("read from replaced fd failed")
        .expect("replaced fd is empty");
    // Don't use assert_eq! here: on failure it would dump both binaries.
    assert!(
        content == expected,
        "replaced fd content differs: got {} bytes, expected {} bytes",
        content.len(),
        expected.len()
    );
    println!(
        "replace: fd {:?} now reads hello_world ({} bytes)",
        target.index(),
        content.len()
    );

    // Out-of-range target index must be rejected.
    let error = source
        .replace(FileDescriptorIndex::new(1_000))
        .expect_err("replace with invalid index should fail");
    assert_eq!(error, ReplaceFdError::InvalidOldFileDescriptorIndex);
    println!("replace: invalid index rejected with {:?}", error);
}
