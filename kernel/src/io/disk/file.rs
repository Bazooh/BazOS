use crate::io::disk::inode::VirtDiskFileNode;
use crate::{
    fs::device::BlockDevice,
    io::{device::BLOCK_DEVICE, disk::BLOCK_SIZE},
};
use common::opened_resource::Resource;
use common::syscall_errors::read::ReadError;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct VirtDiskFile {
    inode: VirtDiskFileNode,
    current_block_index: AtomicUsize,
}

impl VirtDiskFile {
    pub fn new(inode: VirtDiskFileNode) -> Result<VirtDiskFile, ()> {
        Ok(VirtDiskFile {
            inode,
            current_block_index: AtomicUsize::new(0),
        })
    }
}

impl Resource for VirtDiskFile {
    fn read_into(&self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        let block_index = self.current_block_index.fetch_add(1, Ordering::Relaxed);
        let Some(block) = self.inode.data_block(block_index) else {
            return Ok(0);
        };
        BLOCK_DEVICE.lock().read(block, buffer);
        Ok(BLOCK_SIZE)
    }

    fn write(&self, _data: &[u8]) -> Result<(), ()> {
        todo!()
    }

    fn name(&self) -> &str {
        self.inode.name()
    }
}

impl Drop for VirtDiskFile {
    fn drop(&mut self) {}
}
