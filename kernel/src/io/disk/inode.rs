use alloc::collections::BTreeMap;
use core::str::from_utf8;

use crate::fs::device::BlockDevice;
use crate::io::device::BLOCK_DEVICE;
use crate::io::disk::{BLOCK_SIZE, driver::BlockId};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub enum VirtDiskINode {
    File(VirtDiskFileNode),
    Directory(VirtDiskDirNode),
}

#[derive(Debug)]
pub struct VirtDiskFileNode {
    name: String,
    blocks: Vec<BlockId>,
    size: u64,
}

#[derive(Debug)]
pub struct VirtDiskDirNode {
    name: String,
    blocks: BTreeMap<String, BlockId>,
}

type Block = [u8; BLOCK_SIZE];

type BlockU64 = [u64; BLOCK_SIZE / 8];

trait Constructor {
    fn new() -> Self;
}

impl Constructor for Block {
    fn new() -> Block {
        [0; BLOCK_SIZE]
    }
}

trait U64BlockConverter {
    fn as_u64_block(&self) -> &BlockU64;

    fn as_mut_u64_block(&mut self) -> &mut BlockU64;
}

impl U64BlockConverter for Block {
    fn as_u64_block(&self) -> &BlockU64 {
        let (prefix, buffer, suffix) = unsafe { self.align_to::<u64>() };
        assert_eq!(prefix.len(), 0);
        assert_eq!(suffix.len(), 0);
        buffer.as_array().unwrap()
    }

    fn as_mut_u64_block(&mut self) -> &mut BlockU64 {
        let (prefix, buffer, suffix) = unsafe { self.align_to_mut::<u64>() };
        assert_eq!(prefix.len(), 0);
        assert_eq!(suffix.len(), 0);
        buffer.as_mut_array().unwrap()
    }
}

impl VirtDiskINode {
    pub fn read(block_id: BlockId) -> Result<VirtDiskINode, String> {
        let mut buffer = Block::new();
        BLOCK_DEVICE.lock().read(block_id, &mut buffer);
        Self::read_from(&buffer)
    }

    pub fn read_name(block_id: BlockId) -> Result<String, String> {
        let mut buffer = Block::new();

        BLOCK_DEVICE.lock().read(block_id, &mut buffer);

        match buffer[0] {
            0 => Ok(VirtDiskFileNode::read_name(&buffer).ok_or("Invalid name")?),
            1 => Ok(VirtDiskDirNode::read_name(&buffer).ok_or("Invalid name")?),
            _ => Err(String::from("Invalid inode type")),
        }
    }

    fn read_from(buffer: &Block) -> Result<VirtDiskINode, String> {
        if buffer[1] != 0 {
            return Err(String::from("File is locked"));
        }

        match buffer[0] {
            0 => Ok(VirtDiskINode::File(
                VirtDiskFileNode::read_from(buffer).ok_or("Invalid buffer")?,
            )),
            1 => Ok(VirtDiskINode::Directory(
                VirtDiskDirNode::read_from(buffer).ok_or("Invalid buffer")?,
            )),
            _ => Err(String::from("Invalid inode type")),
        }
    }
}

impl VirtDiskFileNode {
    fn new(name: &str) -> Option<Self> {
        if name.as_bytes().len() > 20 {
            return None;
        }

        Some(VirtDiskFileNode {
            name: String::from(name),
            blocks: Vec::new(),
            size: 0,
        })
    }

    fn read_from(buffer: &Block) -> Option<Self> {
        let size = u64::from_le_bytes(buffer[4..12].try_into().unwrap());
        Some(VirtDiskFileNode {
            name: Self::read_name(buffer)?,
            blocks: blocks(&buffer, BlockId::data),
            size,
        })
    }

    fn read_name(buffer: &Block) -> Option<String> {
        Some(String::from(name(buffer, 12, 32)?))
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn data_blocks(&self) -> impl Iterator<Item = BlockId> {
        self.blocks.iter().copied()
    }

    pub fn data_block(&self, index: usize) -> Option<BlockId> {
        self.blocks.get(index).copied()
    }
}

impl VirtDiskDirNode {
    fn new(name: &str) -> Option<Self> {
        if name.as_bytes().len() > 28 {
            return None;
        }

        Some(VirtDiskDirNode {
            name: String::from(name),
            blocks: BTreeMap::new(),
        })
    }
    fn read_from(buffer: &Block) -> Option<Self> {
        let mut blocks_map = BTreeMap::new();
        for block in blocks(buffer, BlockId::inode) {
            core::assert_matches!(block, BlockId::INode(..));
            let name = VirtDiskINode::read_name(block).ok()?;
            blocks_map.insert(name, block);
        }

        Some(VirtDiskDirNode {
            name: Self::read_name(buffer)?,
            blocks: blocks_map,
        })
    }

    fn read_name(buffer: &Block) -> Option<String> {
        Some(String::from(name(buffer, 4, 32)?))
    }

    fn files(&self) -> impl Iterator<Item = BlockId> {
        self.blocks.values().copied()
    }

    pub fn read(&self, name: &str) -> Option<VirtDiskINode> {
        let block_id = *self.blocks.get(name)?;
        VirtDiskINode::read(block_id).ok()
    }
}
fn name(buffer: &Block, start: usize, end: usize) -> Option<&str> {
    let raw = &buffer[start..end];
    let len = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    from_utf8(&raw[..len]).ok()
}

fn blocks(buffer: &Block, block_creator: impl Fn(u64) -> BlockId) -> Vec<BlockId> {
    let mut blocks = Vec::new();
    add_blocks(&mut blocks, buffer, 4, block_creator);
    blocks
}

fn add_blocks(
    blocks: &mut Vec<BlockId>,
    buffer: &Block,
    start: usize,
    block_creator: impl Fn(u64) -> BlockId,
) {
    let buffer = buffer.as_u64_block();
    for block in start..63 {
        let address = buffer[block];
        if address == 0 {
            return;
        }
        blocks.push(block_creator(address));
    }
    if buffer[63] != 0 {
        let mut new_buffer = [0u8; BLOCK_SIZE];
        BLOCK_DEVICE
            .lock()
            .read(BlockId::inode(buffer[63]), &mut new_buffer);
        add_blocks(blocks, &new_buffer, 0, block_creator);
    }
}
