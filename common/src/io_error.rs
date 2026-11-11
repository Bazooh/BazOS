use crate::path::Path;
use alloc::string::String;

#[derive(Debug)]
pub enum IOError {
    AlreadyExists,
    DirectoryFull,
    DoesNotExist(Path),
    InvalidPath,
    NameTooLong,
    NotEnoughSpace,
    NotADirectory(String),
    CannotOpenADirectory,
    AlreadyOpened,
    EmptyFile,
}
