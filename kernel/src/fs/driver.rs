use alloc::string::String;

use crate::fs::{file::File, path::Path};

#[derive(Debug)]
pub enum IOError {
    AlreadyExists,
    DirectoryFull,
    DoesNotExist,
    InvalidPath,
    NameTooLong,
    NoSpace,
    NotADirectory(String),
    CannotOpenADirectory,
}

pub trait DiskDriver {
    fn create(&self, path: Path) -> Result<impl File, IOError>;
    fn open(&self, path: Path) -> Result<impl File, IOError>;
}
