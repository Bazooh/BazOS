use crate::io::disk::{BLOCK_SIZE, MAGIC_NUMBER, N_INODE_BITMAPS, driver::BlockId};

#[derive(Debug)]
pub struct VirtDiskHeader;

impl VirtDiskHeader {
    pub fn from_buffer(buffer: [u8; BLOCK_SIZE]) -> Option<Self> {
        if usize::from_le_bytes(buffer[0..8].try_into().unwrap()) != MAGIC_NUMBER {
            return None;
        }

        Some(VirtDiskHeader)
    }

    pub fn inodes_bitmap_indexes(&self) -> impl Iterator<Item = BlockId> {
        (0..N_INODE_BITMAPS).map(|x| BlockId::inode_bitmap(x))
    }
}
