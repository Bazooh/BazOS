use crate::Block;
use crate::disk::BLOCK_SIZE;
use std::fmt::Debug;

pub struct DataNode {
    pub data: [u8; BLOCK_SIZE],
    pub address: u64,
}

impl Debug for DataNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataNode")
            .field("address", &self.address)
            .finish()
    }
}

#[derive(Debug)]
pub enum INode {
    Dir(HeaderDirINode),
    File(HeaderFileINode),
    SecondaryDir(SecondaryDirINode),
    SecondaryFile(SecondaryFileINode),
}

trait DirINode {
    fn next(&self) -> Option<&INode>;

    fn inodes(&self) -> &Vec<INode>;

    fn collect_nodes<'a>(
        &'a self,
        inodes: &mut Vec<&'a INode>,
        data_nodes: &mut Vec<&'a DataNode>,
    ) {
        for inode in self.inodes() {
            inode.collect_nodes(inodes, data_nodes);
        }
        if let Some(next) = self.next() {
            next.collect_nodes(inodes, data_nodes);
        }
    }
}

impl DirINode for HeaderDirINode {
    fn next(&self) -> Option<&INode> {
        self.next.as_deref()
    }

    fn inodes(&self) -> &Vec<INode> {
        &self.inodes
    }
}

impl DirINode for SecondaryDirINode {
    fn next(&self) -> Option<&INode> {
        self.next.as_deref()
    }

    fn inodes(&self) -> &Vec<INode> {
        &self.inodes
    }
}

trait FileINode {
    fn next(&self) -> Option<&INode>;

    fn data_nodes(&self) -> &Vec<DataNode>;

    fn collect_nodes<'a>(
        &'a self,
        inodes: &mut Vec<&'a INode>,
        data_nodes: &mut Vec<&'a DataNode>,
    ) {
        data_nodes.extend(self.data_nodes());
        if let Some(next) = self.next() {
            next.collect_nodes(inodes, data_nodes);
        }
    }
}

impl FileINode for HeaderFileINode {
    fn next(&self) -> Option<&INode> {
        self.next.as_deref()
    }

    fn data_nodes(&self) -> &Vec<DataNode> {
        &self.data_nodes
    }
}

impl FileINode for SecondaryFileINode {
    fn next(&self) -> Option<&INode> {
        self.next.as_deref()
    }

    fn data_nodes(&self) -> &Vec<DataNode> {
        &self.data_nodes
    }
}

#[derive(Debug)]
pub struct HeaderDirINode {
    pub next: Option<Box<INode>>,
    pub inodes: Vec<INode>,
    pub address: u64,
    pub name: String,
}

#[derive(Debug)]
pub struct SecondaryDirINode {
    pub next: Option<Box<INode>>,
    pub inodes: Vec<INode>,
    pub address: u64,
}

#[derive(Debug)]
pub struct SecondaryFileINode {
    pub next: Option<Box<INode>>,
    pub data_nodes: Vec<DataNode>,
    pub address: u64,
}

#[derive(Debug)]
pub struct HeaderFileINode {
    pub next: Option<Box<INode>>,
    pub data_nodes: Vec<DataNode>,
    pub address: u64,
    pub length: u64,
    pub name: String,
}

impl INode {
    pub fn nodes(&self) -> (Vec<&INode>, Vec<&DataNode>) {
        let mut inodes = Vec::new();
        let mut data_nodes = Vec::new();
        self.collect_nodes(&mut inodes, &mut data_nodes);
        (inodes, data_nodes)
    }

    pub fn addr(&self) -> u64 {
        match self {
            INode::Dir(x) => x.address,
            INode::File(x) => x.address,
            INode::SecondaryDir(x) => x.address,
            INode::SecondaryFile(x) => x.address,
        }
    }

    pub fn block(&self) -> Block {
        match self {
            INode::Dir(x) => x.block(),
            INode::File(x) => x.block(),
            INode::SecondaryDir(x) => x.block(),
            INode::SecondaryFile(x) => x.block(),
        }
    }

    fn collect_nodes<'a>(
        &'a self,
        inodes: &mut Vec<&'a INode>,
        data_nodes: &mut Vec<&'a DataNode>,
    ) {
        inodes.push(&self);
        match self {
            Self::File(node) => node.collect_nodes(inodes, data_nodes),
            Self::SecondaryFile(node) => node.collect_nodes(inodes, data_nodes),
            Self::Dir(node) => node.collect_nodes(inodes, data_nodes),
            Self::SecondaryDir(node) => node.collect_nodes(inodes, data_nodes),
        }
    }
}

impl HeaderDirINode {
    fn block(&self) -> Block {
        assert!(self.name.len() <= 28);
        assert!(self.inodes.len() <= 59);

        let mut buffer = [0u8; BLOCK_SIZE];
        buffer[0] = 1;
        buffer[4..self.name.len() + 4].copy_from_slice(self.name.as_bytes());
        for (i, inode) in self.inodes.iter().enumerate() {
            buffer[32 + i * 8..40 + i * 8].copy_from_slice(&inode.addr().to_le_bytes());
        }
        if let Some(value) = &self.next {
            buffer[504..512].copy_from_slice(&value.addr().to_le_bytes());
        }
        buffer
    }
}

impl SecondaryDirINode {
    fn block(&self) -> Block {
        assert!(self.inodes.len() <= 63);

        let mut buffer = [0u8; BLOCK_SIZE];
        for (i, inode) in self.inodes.iter().enumerate() {
            buffer[i * 8..(i + 1) * 8].copy_from_slice(&inode.addr().to_le_bytes());
        }
        if let Some(value) = &self.next {
            buffer[504..512].copy_from_slice(&value.addr().to_le_bytes());
        }
        buffer
    }
}

impl HeaderFileINode {
    fn block(&self) -> Block {
        assert!(self.name.len() <= 28);
        assert!(self.data_nodes.len() <= 59);

        let mut buffer = [0u8; BLOCK_SIZE];
        buffer[4..12].copy_from_slice(&self.length.to_le_bytes());
        buffer[12..self.name.len() + 12].copy_from_slice(self.name.as_bytes());
        for (i, data_node) in self.data_nodes.iter().enumerate() {
            buffer[32 + i * 8..40 + i * 8].copy_from_slice(&data_node.address.to_le_bytes());
        }
        if let Some(value) = &self.next {
            buffer[504..512].copy_from_slice(&value.addr().to_le_bytes());
        }
        buffer
    }
}

impl SecondaryFileINode {
    fn block(&self) -> Block {
        assert!(self.data_nodes.len() <= 63);

        let mut buffer: Block = [0u8; BLOCK_SIZE];
        for (i, data_node) in self.data_nodes.iter().enumerate() {
            buffer[i * 8..(i + 1) * 8].copy_from_slice(&data_node.address.to_le_bytes());
        }
        if let Some(value) = &self.next {
            buffer[504..512].copy_from_slice(&value.addr().to_le_bytes());
        }
        buffer
    }
}
