use crate::io::disk::inode::VirtDiskFileNode;
use crate::{
    fs::device::BlockDevice,
    io::{device::BLOCK_DEVICE, disk::BLOCK_SIZE},
};
use common::opened_resource::OpenedResource;
use common::syscall_errors::read::ReadError;

pub struct VirtDiskFile {
    inode: VirtDiskFileNode,
    current_block_index: usize,
}

impl VirtDiskFile {
    pub fn new(inode: VirtDiskFileNode) -> Result<VirtDiskFile, ()> {
        Ok(VirtDiskFile {
            inode,
            current_block_index: 0,
        })
    }
}

impl OpenedResource for VirtDiskFile {
    fn read_into(&mut self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        let Some(block) = self.inode.data_block(self.current_block_index) else {
            return Ok(0);
        };
        BLOCK_DEVICE.lock().read(block, buffer);
        self.current_block_index += 1;
        Ok(BLOCK_SIZE)
    }

    fn write(&mut self, _data: &[u8]) -> Result<(), ()> {
        todo!()
    }

    fn name(&self) -> &str {
        self.inode.name()
    }
}

impl Drop for VirtDiskFile {
    fn drop(&mut self) {}
}
