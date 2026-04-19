pub trait BlockDevice {
    fn read(&mut self, block_id: usize, buf: &mut [u8]);

    fn write(&mut self, block_id: usize, block: &[u8]);
}
