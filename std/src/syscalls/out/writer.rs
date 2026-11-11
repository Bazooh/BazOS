use crate::syscalls::fs::write::write_all;
use common::fs::FileDescriptorIndex;
use core::fmt::{Error, Result, Write};

pub(crate) struct Writer {
    fd_index: FileDescriptorIndex,
}

impl Writer {
    pub(crate) fn stdout() -> Self {
        Self {
            fd_index: FileDescriptorIndex::STDOUT,
        }
    }

    pub(crate) fn stderr() -> Self {
        Self {
            fd_index: FileDescriptorIndex::STDERR,
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, string: &str) -> Result {
        write_all(self.fd_index, string.as_bytes()).map_err(|_| Error)
    }
}
