use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    iter::once,
};

use crate::disk::{BLOCK_SIZE, path::Path};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Node {
    File(FileNode),
    Dir(DirNode),
}

#[derive(Debug, Eq)]
pub struct FileNode {
    name: String,
    path: Path,
    content: Box<[u8]>,
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
struct DirNode {
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
        FileNode::new(path, content).map(|file_node| Node::File(file_node))
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
            Node::File(..) => return Box::new(once(self)),
            Node::Dir(node) => {
                return Box::new(
                    once(self).chain(node.children.values().flat_map(|child| child.iter())),
                );
            }
        }
    }

    pub fn header(&self, inode_map: &HashMap<&Node, (usize, usize)>) -> [u8; BLOCK_SIZE] {
        match self {
            Node::File(node) => node.header(inode_map.get(self).unwrap().1),
            Node::Dir(node) => node.header(inode_map),
        }
    }
}

impl FileNode {
    fn new(path: Path, content: Box<[u8]>) -> Option<Self> {
        let name = String::from(path.split().pop()?);
        assert!(name.len() <= 20);
        return Some(FileNode {
            path,
            name,
            content,
        });
    }

    pub fn data_chunks(&self) -> impl Iterator<Item = &[u8]> {
        self.content.as_ref().chunks(BLOCK_SIZE)
    }

    pub fn n_data_chunks(&self) -> usize {
        self.content.len().div_ceil(BLOCK_SIZE)
    }

    fn header(&self, data_offset: usize) -> [u8; BLOCK_SIZE] {
        let mut buffer = [0u8; BLOCK_SIZE];
        buffer[4..12].copy_from_slice(&self.content.len().to_le_bytes());
        buffer[12..self.name.len() + 12].copy_from_slice(self.name.as_bytes());
        assert!(self.n_data_chunks() <= 60);
        for i in 0..self.n_data_chunks() {
            buffer[32 + i * 8..40 + i * 8].copy_from_slice(&(data_offset + i).to_le_bytes());
        }
        buffer
    }
}

impl DirNode {
    fn new(path: Path) -> Option<Self> {
        let name = String::from(path.split().pop()?);
        assert!(name.len() <= 28);
        return Some(DirNode {
            path,
            name,
            children: HashMap::new(),
        });
    }

    fn root() -> Self {
        return DirNode {
            name: String::from("$root"),
            path: Path::new(String::from("")),
            children: HashMap::new(),
        };
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

    fn header(&self, inode_map: &HashMap<&Node, (usize, usize)>) -> [u8; BLOCK_SIZE] {
        let mut buffer = [0u8; BLOCK_SIZE];
        buffer[0] = 1;
        buffer[4..self.name.len() + 4].copy_from_slice(self.name.as_bytes());
        for (i, (_, node)) in self.children.iter().enumerate() {
            let (inode_index, _) = inode_map[node];
            buffer[32 + i * 8..40 + i * 8].copy_from_slice(&inode_index.to_le_bytes());
        }
        buffer
    }
}
