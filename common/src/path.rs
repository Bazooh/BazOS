use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct Path {
    path: String,
}

impl Path {
    pub fn new(path: &str) -> Result<Path, ()> {
        Ok(Path {
            path: String::from(path),
        })
    }

    pub fn split(&self) -> Vec<&str> {
        self.path.split('/').collect()
    }
}
