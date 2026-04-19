use alloc::{string::String, vec::Vec};

pub struct Path {
    path: String,
}

impl Path {
    pub fn new(path: &str) -> Self {
        Path {
            path: String::from(path),
        }
    }

    pub fn split(&self) -> Vec<&str> {
        self.path.split('/').collect()
    }
}
