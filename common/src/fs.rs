#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FileDescriptorIndex(u64);

impl FileDescriptorIndex {
    pub const STDIN: FileDescriptorIndex = FileDescriptorIndex(0);
    pub const STDOUT: FileDescriptorIndex = FileDescriptorIndex(1);
    pub const STDERR: FileDescriptorIndex = FileDescriptorIndex(2);

    pub fn new(value: u64) -> FileDescriptorIndex {
        FileDescriptorIndex(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
