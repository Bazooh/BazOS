use common::syscall::{SyscallNumber, syscall};
use x86_64::VirtAddr;

pub fn mmap(n_pages: u64) -> Option<VirtAddr> {
    if n_pages == 0 {
        return None;
    }
    let result = syscall(SyscallNumber::MMap, n_pages, 0, 0, 0);
    if result == 0 {
        return None;
    }

    let addr = u64::from_ne_bytes(result.to_ne_bytes());
    assert_eq!(((addr << 16) as i64 >> 16) as u64, addr);

    Some(VirtAddr::try_new(addr).unwrap())
}
