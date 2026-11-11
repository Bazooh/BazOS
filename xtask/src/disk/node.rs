use crate::disk::inodes::{
    DataNode, HeaderDirINode, HeaderFileINode, INode, SecondaryDirINode, SecondaryFileINode,
};
use crate::disk::{BLOCK_SIZE, path::Path};
use std::fmt::Debug;
use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    iter::once,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Node {
    File(FileNode),
    Dir(DirNode),
}

#[derive(Eq)]
pub struct FileNode {
    name: String,
    path: Path,
    content: Box<[u8]>,
}

impl fmt::Debug for FileNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileNode")
            .field("name", &self.name)
            .field("path", &self.path)
            .finish()
    }
}

impl PartialEq for FileNode {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Hash for FileNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, Eq)]
pub struct DirNode {
    name: String,
    path: Path,
    children: HashMap<String, Node>,
}

impl PartialEq for DirNode {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Hash for DirNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug)]
pub enum FileCreationError {
    AlreadyExist,
    InvalidPath,
    NotADirectory(String),
}

impl Node {
    pub fn file(path: Path, content: Box<[u8]>) -> Option<Self> {
        Some(Node::File(FileNode::new(path, content)?))
    }

    pub fn dir(path: Path) -> Option<Self> {
        DirNode::new(path).map(|dir_node| Node::Dir(dir_node))
    }

    pub fn root_dir() -> Self {
        Node::Dir(DirNode::root())
    }

    fn add_node(&mut self, node: Node, local_path: Path) -> Result<(), FileCreationError> {
        match self {
            Node::File(..) => Err(FileCreationError::NotADirectory(self.name())),
            Node::Dir(dir_node) => dir_node.add_node(node, local_path),
        }
    }

    pub fn add_file(&mut self, path: &str, content: Box<[u8]>) -> Result<(), FileCreationError> {
        let path = Path::new(String::from(path));
        self.add_node(
            Node::file(path.clone(), content).ok_or(FileCreationError::InvalidPath)?,
            path,
        )
    }

    pub fn add_dir(&mut self, path: &str) -> Result<(), FileCreationError> {
        let path = Path::new(String::from(path));
        self.add_node(
            Node::dir(path.clone()).ok_or(FileCreationError::InvalidPath)?,
            path,
        )
    }

    pub fn name(&self) -> String {
        match self {
            Node::File(node) => node.name.clone(),
            Node::Dir(node) => node.name.clone(),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &Node> + '_> {
        match self {
            Node::File(..) => Box::new(once(self)),
            Node::Dir(node) => {
                Box::new(once(self).chain(node.children.values().flat_map(|child| child.iter())))
            }
        }
    }

    pub fn to_inode(&self, inode_count: &mut u64, data_node_count: &mut u64) -> INode {
        match self {
            Node::File(node) => node.to_inode(inode_count, data_node_count),
            Node::Dir(node) => node.to_inode(inode_count, data_node_count),
        }
    }
}

impl FileNode {
    fn new(path: Path, content: Box<[u8]>) -> Option<Self> {
        let name = String::from(path.split().pop()?);
        assert!(name.len() <= 20);
        Some(FileNode {
            path,
            name,
            content,
        })
    }

    pub fn data_chunks(&self) -> impl Iterator<Item = &[u8]> {
        self.content.as_ref().chunks(BLOCK_SIZE)
    }

    fn data_nodes<'a>(
        chunks: &mut impl Iterator<Item = &'a [u8]>,
        data_node_count: &mut u64,
        count: usize,
    ) -> Vec<DataNode> {
        let mut data_nodes = Vec::with_capacity(count);
        while data_nodes.len() < count
            && let Some(chunk) = chunks.next()
        {
            let address = *data_node_count;
            *data_node_count += 1;
            let mut data = [0u8; BLOCK_SIZE];
            data[..chunk.len()].copy_from_slice(chunk);
            data_nodes.push(DataNode { data, address })
        }
        data_nodes
    }

    fn secondary_header_node<'a>(
        &self,
        mut chunks: impl Iterator<Item = &'a [u8]>,
        inode_count: &mut u64,
        data_node_count: &mut u64,
    ) -> Option<Box<INode>> {
        let data_nodes = Self::data_nodes(&mut chunks, data_node_count, 63);
        if data_nodes.is_empty() {
            return None;
        }
        let address = *inode_count;
        *inode_count += 1;
        Some(Box::new(INode::SecondaryFile(SecondaryFileINode {
            next: self.secondary_header_node(chunks, inode_count, data_node_count),
            data_nodes,
            address,
        })))
    }

    fn header_node<'a>(
        &self,
        mut chunks: impl Iterator<Item = &'a [u8]>,
        inode_count: &mut u64,
        data_node_count: &mut u64,
    ) -> HeaderFileINode {
        let address = *inode_count;
        *inode_count += 1;

        let data_nodes = Self::data_nodes(&mut chunks, data_node_count, 59);

        HeaderFileINode {
            next: self.secondary_header_node(chunks, inode_count, data_node_count),
            data_nodes,
            address,
            length: self.content.len() as u64,
            name: self.name.clone(),
        }
    }

    fn to_inode(&self, inode_count: &mut u64, data_node_count: &mut u64) -> INode {
        INode::File(self.header_node(self.data_chunks(), inode_count, data_node_count))
    }
}

impl DirNode {
    fn new(path: Path) -> Option<Self> {
        let name = String::from(path.split().pop()?);
        assert!(name.len() <= 28);
        Some(DirNode {
            path,
            name,
            children: HashMap::new(),
        })
    }

    fn root() -> Self {
        DirNode {
            name: String::from("$root"),
            path: Path::new(String::from("")),
            children: HashMap::new(),
        }
    }

    fn add_node(&mut self, node: Node, local_path: Path) -> Result<(), FileCreationError> {
        let parts = local_path.split();
        let (first, rest) = parts.split_at(1);
        let name = String::from(first[0]);

        if rest.is_empty() {
            if self.children.contains_key(&name) {
                return Err(FileCreationError::AlreadyExist);
            }
            self.children.insert(name, node);
            return Ok(());
        }

        let next_dir = self
            .children
            .entry(name.clone())
            .or_insert_with(|| Node::dir(self.path.add(name)).unwrap());

        next_dir.add_node(node, Path::from_parts(rest))
    }

    fn inodes<'a>(
        children: &mut impl Iterator<Item = &'a Node>,
        inode_count: &mut u64,
        data_node_count: &mut u64,
        count: usize,
    ) -> Vec<INode> {
        let mut inodes = Vec::with_capacity(count);
        while inodes.len() < count
            && let Some(child) = children.next()
        {
            inodes.push(child.to_inode(inode_count, data_node_count));
        }
        inodes
    }

    fn secondary_header_node<'a>(
        &self,
        mut children: impl Iterator<Item = &'a Node>,
        inode_count: &mut u64,
        data_node_count: &mut u64,
    ) -> Option<Box<INode>> {
        let inodes = Self::inodes(&mut children, inode_count, data_node_count, 63);
        if inodes.is_empty() {
            return None;
        }
        let address = *inode_count;
        *inode_count += 1;
        Some(Box::new(INode::SecondaryDir(SecondaryDirINode {
            next: self.secondary_header_node(children, inode_count, data_node_count),
            inodes,
            address,
        })))
    }

    fn header_node<'a>(
        &self,
        mut children: impl Iterator<Item = &'a Node>,
        inode_count: &mut u64,
        data_node_count: &mut u64,
    ) -> HeaderDirINode {
        let address = *inode_count;
        *inode_count += 1;

        let inodes = Self::inodes(&mut children, inode_count, data_node_count, 59);

        HeaderDirINode {
            next: self.secondary_header_node(children, inode_count, data_node_count),
            address,
            inodes,
            name: self.name.clone(),
        }
    }

    fn to_inode(&self, inode_count: &mut u64, data_node_count: &mut u64) -> INode {
        INode::Dir(self.header_node(self.children.values(), inode_count, data_node_count))
    }
}
