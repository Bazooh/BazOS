#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FileDescriptor(u64);

impl FileDescriptor {
    pub fn new(value: u64) -> FileDescriptor {
        FileDescriptor(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
