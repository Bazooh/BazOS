use core::slice::from_raw_parts;

use crate::io::disk::BLOCK_SIZE;

pub struct VirtDiskBitmap<'a> {
    bitmap: &'a [u64],
}

impl<'a> VirtDiskBitmap<'a> {
    pub fn from_buffer(buffer: &'a [u8; BLOCK_SIZE]) -> Self {
        let bitmap = unsafe { from_raw_parts(buffer.as_ptr() as *const u64, BLOCK_SIZE / 8) };
        VirtDiskBitmap { bitmap }
    }

    pub fn find_zero(&self) -> Option<usize> {
        for index in 0..self.bitmap.len() {
            let bit_index = self.bitmap[index].trailing_ones() as usize;
            if bit_index != u64::BITS as usize {
                return Some(index * u64::BITS as usize + bit_index);
            }
        }
        None
    }
}
