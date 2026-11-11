use std::fs::{OpenOptions, read};
use std::io::Write;
use std::process::Command;

use crate::disk::BLOCK_SIZE;
use crate::disk::node::Node;

mod disk;

fn run(cmd: &mut Command) {
    let status = cmd.status().expect("failed to run");
    assert!(status.success());
}

const BINARIES: [&str; 3] = ["hello_world", "terminal", "commands/echo"];
const DISK_PATH: &str = "disk/disk.img";

const MAGIC_NUMBER: u64 = 0xBA2_05;

const N_DATA_BITMAPS: usize = 16;
const N_INODE_BITMAPS: usize = 1;
const N_INODES: usize = N_INODE_BITMAPS * 8 * BLOCK_SIZE;
const N_DATA_BLOCKS: usize = N_DATA_BITMAPS * 8 * BLOCK_SIZE;
const NUMBER_BLOCKS: usize = 1 + N_DATA_BITMAPS + N_INODE_BITMAPS + N_INODES + N_DATA_BLOCKS;

const DATA_BITMAP_OFFSET: usize = 1;
const INODE_BITMAP_OFFSET: usize = DATA_BITMAP_OFFSET + N_DATA_BITMAPS;
const INODE_OFFSET: usize = INODE_BITMAP_OFFSET + N_INODE_BITMAPS;
const DATA_OFFSET: usize = INODE_OFFSET + N_INODES;

type Block = [u8; BLOCK_SIZE];

fn main() {
    std::fs::create_dir_all("disk").unwrap();

    let mut root = Node::root_dir();
    for name in BINARIES {
        run(Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(format!("user/{}", name))
            .env("RUSTFLAGS", "-C linker=/Users/aymeric/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/rust-lld -C link-arg=-Tlinker.ld"));
        let content = read(format!(
            "target/x86_64-BazOS-user/release/{}",
            name.split('/').last().unwrap()
        ))
        .expect("Failed to read file");

        // TODO: implement directories
        root.add_file(name.replace('/', "_").as_str(), content.into_boxed_slice())
            .unwrap();
    }

    let root = root.to_inode(&mut 0, &mut 1);
    let (inodes, data_nodes) = root.nodes();
    assert!(inodes.len() <= N_INODES);

    let inode_bitmap = {
        let mut buffer = Box::new([0u8; BLOCK_SIZE * N_INODE_BITMAPS]);
        for inode in &inodes {
            let addr = inode.addr() as usize;
            buffer[addr / 8] |= (1 << (u8::BITS - 1)) >> (addr % 8);
        }
        buffer
    };
    let data_bitmap = {
        let mut buffers = Box::new([0u8; BLOCK_SIZE * N_DATA_BITMAPS]);
        for data_node in &data_nodes {
            let addr = data_node.address as usize;
            buffers[addr / 8] |= (1 << (u8::BITS - 1)) >> (addr % 8);
        }
        buffers
    };

    let mut buffer = vec![0; NUMBER_BLOCKS * BLOCK_SIZE];

    buffer[..8].copy_from_slice(&MAGIC_NUMBER.to_le_bytes());
    buffer[8..16].copy_from_slice(&N_DATA_BITMAPS.to_le_bytes());
    buffer[16..24].copy_from_slice(&N_INODE_BITMAPS.to_le_bytes());
    buffer[24..32].copy_from_slice(&N_INODES.to_le_bytes());
    buffer[32..40].copy_from_slice(&N_DATA_BLOCKS.to_le_bytes());

    buffer[DATA_BITMAP_OFFSET * BLOCK_SIZE..INODE_BITMAP_OFFSET * BLOCK_SIZE]
        .copy_from_slice(&data_bitmap[..]);
    buffer[INODE_BITMAP_OFFSET * BLOCK_SIZE..INODE_OFFSET * BLOCK_SIZE]
        .copy_from_slice(&inode_bitmap[..]);

    for inode in inodes {
        let content = inode.block();
        let offset = (INODE_OFFSET + inode.addr() as usize) * BLOCK_SIZE;
        buffer[offset..offset + content.len()].copy_from_slice(content.as_slice());
    }

    for data_node in data_nodes {
        let content = data_node.data;
        let offset = (DATA_OFFSET + data_node.address as usize) * BLOCK_SIZE;
        buffer[offset..offset + content.len()].copy_from_slice(content.as_slice());
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(DISK_PATH)
        .unwrap();

    file.write(buffer.as_slice()).unwrap();
}
