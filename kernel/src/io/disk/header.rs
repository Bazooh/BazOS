use crate::io::disk::{BLOCK_SIZE, MAGIC_NUMBER};

#[derive(Debug)]
pub struct VirtDiskHeader;

impl VirtDiskHeader {
    pub fn from_buffer(buffer: [u8; BLOCK_SIZE]) -> Option<Self> {
        if u64::from_le_bytes(buffer[0..8].try_into().unwrap()) != MAGIC_NUMBER {
            return None;
        }

        Some(VirtDiskHeader)
    }
}
