use alloc::{boxed::Box, string::String, vec::Vec};

use crate::{
    fs::{device::BlockDevice, file::File},
    io::{
        device::BLOCK_DEVICE,
        disk::{
            BLOCK_SIZE,
            driver::{DISK_DRIVER, INodeIndex},
            inode::VirtDiskINode,
        },
    },
};

pub struct VirtDiskFile {
    name: String,
    inode_index: INodeIndex,
}

impl VirtDiskFile {
    pub fn new(name: String, inode_index: INodeIndex) -> Self {
        Self { name, inode_index }
    }
}

impl File for VirtDiskFile {
    fn read(&self) -> Box<[u8]> {
        let VirtDiskINode::File(inode) = DISK_DRIVER
            .try_get()
            .unwrap()
            .get_inode(self.inode_index)
            .unwrap()
        else {
            panic!("File is not a file")
        };

        let mut content = Vec::with_capacity(inode.size());
        let mut buffer = [0u8; BLOCK_SIZE];
        for data in inode.data_blocks() {
            BLOCK_DEVICE.lock().read(data.as_usize(), &mut buffer);
            content.extend_from_slice(&buffer);
        }

        content.into_boxed_slice()
    }

    fn write(&mut self, _data: &[u8]) {}

    fn close(&mut self) {}

    fn name(&self) -> &str {
        &self.name
    }
}
