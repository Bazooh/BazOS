mod bitmap;
pub mod driver;
mod file;
mod header;
mod inode;

const MAGIC_NUMBER: usize = 0xBA2_05;
const BLOCK_SIZE: usize = 512;

const N_DATA_BITMAPS: usize = 16;
const N_INODE_BITMAPS: usize = 1;
const N_INODES: usize = N_INODE_BITMAPS * 8 * BLOCK_SIZE;
const N_DATA_BLOCKS: usize = N_DATA_BITMAPS * 8 * BLOCK_SIZE;
const NUMBER_BLOCKS: usize = 1 + N_DATA_BITMAPS + N_INODE_BITMAPS + N_INODES + N_DATA_BLOCKS;

const DATA_BITMAP_OFFSET: usize = 1;
const INODE_BITMAP_OFFSET: usize = DATA_BITMAP_OFFSET + N_DATA_BITMAPS;
const INODE_OFFSET: usize = INODE_BITMAP_OFFSET + N_INODE_BITMAPS;
const DATA_OFFSET: usize = INODE_OFFSET + N_INODES;
