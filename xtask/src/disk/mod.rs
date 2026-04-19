use crate::disk::node::Node;

pub mod node;
mod path;

pub const BLOCK_SIZE: usize = 512;

pub struct Disk {
    root: Node,
}

impl Disk {
    pub fn new() -> Self {
        Disk {
            root: Node::root_dir(),
        }
    }
}
