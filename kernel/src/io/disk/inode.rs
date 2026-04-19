use core::str::from_utf8;

use alloc::string::String;

use crate::io::disk::{
    BLOCK_SIZE,
    driver::{BlockId, DISK_DRIVER, INodeIndex},
};

pub enum VirtDiskINode {
    File(VirtDiskFileNode),
    Directory(VirtDiskDirNode),
}

pub struct VirtDiskFileNode {
    buffer: [usize; BLOCK_SIZE / 8],
}

pub struct VirtDiskDirNode {
    buffer: [usize; BLOCK_SIZE / 8],
}

pub fn u8_array_view(buffer: &[usize; BLOCK_SIZE / 8]) -> &[u8; BLOCK_SIZE] {
    let (prefix, buffer, sufix) = unsafe { buffer.align_to::<u8>() };
    assert!(prefix.is_empty());
    assert!(sufix.is_empty());
    buffer.as_array().unwrap()
}

pub fn u8_array_mut_view(buffer: &mut [usize; BLOCK_SIZE / 8]) -> &mut [u8; BLOCK_SIZE] {
    let (prefix, buffer, sufix) = unsafe { buffer.align_to_mut::<u8>() };
    assert!(prefix.is_empty());
    assert!(sufix.is_empty());
    buffer.as_mut_array().unwrap()
}

impl VirtDiskINode {
    pub fn from_buffer(buffer: [usize; BLOCK_SIZE / 8]) -> Result<Self, String> {
        let first_bytes = buffer[0].to_le_bytes();
        if first_bytes[1] != 0 {
            return Err(String::from("File is locked"));
        }

        match first_bytes[0] {
            0 => Ok(VirtDiskINode::File(VirtDiskFileNode::from_buffer(buffer))),
            1 => Ok(VirtDiskINode::Directory(VirtDiskDirNode::from_buffer(
                buffer,
            ))),
            _ => Err(String::from("Invalid inode type")),
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            VirtDiskINode::File(file_node) => file_node.name(),
            VirtDiskINode::Directory(dir_node) => dir_node.name(),
        }
    }

    pub fn buffer(&self) -> &[u8; BLOCK_SIZE] {
        match self {
            VirtDiskINode::File(file_node) => file_node.buffer(),
            VirtDiskINode::Directory(dir_node) => dir_node.buffer(),
        }
    }
}

impl VirtDiskFileNode {
    pub fn new(name: &str) -> Option<Self> {
        let bytes = name.as_bytes();
        if bytes.len() > 20 {
            return None;
        }

        let mut buffer = [0usize; BLOCK_SIZE / 8];
        let view = u8_array_mut_view(&mut buffer);
        view[1] = 1;
        view[12..bytes.len() + 12].copy_from_slice(bytes);
        Some(VirtDiskFileNode { buffer })
    }

    pub fn from_buffer(buffer: [usize; BLOCK_SIZE / 8]) -> Self {
        VirtDiskFileNode { buffer }
    }

    pub fn name(&self) -> Option<&str> {
        let raw = &self.buffer()[12..32];
        let len = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        from_utf8(&raw[..len]).ok()
    }

    pub fn buffer(&self) -> &[u8; BLOCK_SIZE] {
        u8_array_view(&self.buffer)
    }

    pub fn size(&self) -> usize {
        usize::from_le_bytes(self.buffer()[4..12].try_into().unwrap())
    }

    pub fn data_blocks(&self) -> impl Iterator<Item = BlockId> {
        self.buffer[4..]
            .iter()
            .take_while(|block_id| **block_id != 0)
            .map(|block_id| BlockId::raw(*block_id))
    }
}

impl VirtDiskDirNode {
    pub fn from_buffer(buffer: [usize; BLOCK_SIZE / 8]) -> Self {
        VirtDiskDirNode { buffer }
    }

    pub fn name(&self) -> Option<&str> {
        let raw = &self.buffer()[1..32];
        let len = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        from_utf8(&raw[..len]).ok()
    }

    pub fn buffer(&self) -> &[u8; BLOCK_SIZE] {
        u8_array_view(&self.buffer)
    }

    fn buffer_mut(&mut self) -> &mut [u8; BLOCK_SIZE] {
        u8_array_mut_view(&mut self.buffer)
    }

    fn files_inode_index(&self) -> impl Iterator<Item = INodeIndex> {
        self.buffer[4..].iter().filter_map(|inode_index| {
            if *inode_index == 0 {
                None
            } else {
                Some(INodeIndex::new(*inode_index))
            }
        })
    }

    pub fn get(&self, name: &str) -> Option<(INodeIndex, VirtDiskINode)> {
        for inode_index in self.files_inode_index() {
            // TODO: Different exception
            let inode = DISK_DRIVER
                .try_get()
                .unwrap()
                .get_inode(inode_index)
                .unwrap();
            if inode.name()? == name {
                return Some((inode_index, inode));
            }
        }
        None
    }

    pub fn add_file(&mut self, _name: &str, file_inode_index: INodeIndex) -> Option<()> {
        for inode_index in self.buffer[4..].iter_mut() {
            if *inode_index == 0 {
                *inode_index = file_inode_index.as_usize();
                return Some(());
            }
        }

        None
    }
}
