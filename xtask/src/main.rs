use std::collections::HashMap;
use std::fs::{self, OpenOptions, read};
use std::io::Write;
use std::process::Command;

use crate::disk::BLOCK_SIZE;
use crate::disk::node::Node;

mod disk;

fn run(cmd: &mut Command) {
    let status = cmd.status().expect("failed to run");
    assert!(status.success());
}

const BINARIES: [&str; 1] = ["hello_world"];
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

fn print_block(block: &Block) {
    for (i, line) in block.chunks(16).enumerate() {
        println!(
            "{i:02x}0:  {}  |{}|",
            line.iter()
                .map(|x| format!("{x:02x}"))
                .collect::<Vec<String>>()
                .join(" "),
            line.iter()
                .map(|x| match *x {
                    0x20..=0x7e => char::from(*x),
                    _ => '.',
                })
                .collect::<String>()
        );
    }
}

fn main() {
    fs::create_dir_all("disk").unwrap();

    let mut root = Node::root_dir();
    for name in BINARIES {
        run(Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(format!("user/{}", name))
            .env("RUSTFLAGS", "-C linker=/Users/aymeric/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/rust-lld -C link-arg=-Tlinker.ld"));
        let content = read(format!("target/x86_64-BazOS-user/release/{}", name))
            .expect("Failed to read file");
        root.add_file(name, content.into_boxed_slice()).unwrap();
    }

    let mut inodes: HashMap<&Node, (usize, usize)> = HashMap::new();
    let mut data_chunks: Vec<Block> = Vec::new();
    for node in root.iter() {
        inodes.insert(node, (inodes.len(), data_chunks.len() + DATA_OFFSET));
        if let Node::File(file) = node {
            data_chunks.extend(file.data_chunks().map(|chunk| {
                let mut buffer = [0u8; BLOCK_SIZE];
                buffer[..chunk.len()].copy_from_slice(&chunk[..chunk.len()]);
                buffer
            }));
        }
    }

    let inode_bitmap = {
        let mut buffer = Box::new([0u8; BLOCK_SIZE]);
        for (_, (inode_index, _)) in inodes.iter() {
            buffer[inode_index / 8] |= (1 << (u8::BITS - 1)) >> (inode_index % 8);
        }
        buffer
    };
    let data_bitmap = {
        let mut buffers = Box::new([0u8; BLOCK_SIZE * N_DATA_BITMAPS]);
        for (inode, (_, data_offset)) in &inodes {
            if let Node::File(file) = inode {
                for i in 0..file.n_data_chunks() {
                    let index = data_offset + i;
                    buffers[index / 8] |= (1 << (u8::BITS - 1)) >> (index % 8);
                }
            }
        }
        buffers
    };
    let inodes = {
        let mut buffers = Box::new([0u8; BLOCK_SIZE * N_INODES]);
        for (inode, (inode_index, _)) in &inodes {
            buffers[inode_index * BLOCK_SIZE..inode_index * BLOCK_SIZE + BLOCK_SIZE]
                .copy_from_slice(&inode.header(&inodes));
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
    buffer[INODE_OFFSET * BLOCK_SIZE..DATA_OFFSET * BLOCK_SIZE].copy_from_slice(&inodes[..]);

    let vec = data_chunks.into_iter().flatten().collect::<Vec<u8>>();
    buffer[DATA_OFFSET * BLOCK_SIZE..DATA_OFFSET * BLOCK_SIZE + vec.len()]
        .copy_from_slice(vec.as_slice());

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(DISK_PATH)
        .unwrap();

    file.write(buffer.as_slice()).unwrap();
}
