use crate::io::disk::driver::BlockId;

pub trait BlockDevice {
    fn read(&mut self, block_id: BlockId, buf: &mut [u8]);

    fn write(&mut self, block_id: BlockId, block: &[u8]);
}
