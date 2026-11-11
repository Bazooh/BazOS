use common::fs::FileDescriptorIndex;
use common::syscall::SyscallNumber::Write;
use common::syscall::syscall;

pub(crate) fn write_all(fd_index: FileDescriptorIndex, mut data: &[u8]) -> Result<(), ()> {
    while !data.is_empty() {
        let wrote = syscall(
            Write,
            fd_index.as_u64(),
            data.as_ptr() as u64,
            data.len() as u64,
            0,
        );
        if wrote <= 0 {
            return Err(());
        }
        data = &data[wrote as usize..];
    }
    Ok(())
}
