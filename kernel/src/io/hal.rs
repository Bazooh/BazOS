use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

use virtio_drivers::{BufferDirection, Hal};
use x86_64::{PhysAddr, VirtAddr, structures::paging::Translate};

use crate::memory::{HEAP, MEMORY_MAPPER, PAGE_SIZE, to_virtual_address};

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(
        pages: usize,
        _direction: BufferDirection,
    ) -> (virtio_drivers::PhysAddr, NonNull<u8>) {
        assert!(pages > 0);

        let layout = Layout::array::<u8>(pages * PAGE_SIZE).unwrap();
        let virt_ptr = unsafe { HEAP.alloc_zeroed(layout) };
        let phys_addr = MEMORY_MAPPER
            .try_get()
            .expect("Heap not initialized")
            .translate_addr(VirtAddr::from_ptr(virt_ptr))
            .expect("Translation failed");

        (
            phys_addr.as_u64(),
            NonNull::new(virt_ptr).expect("Address is null"),
        )
    }

    unsafe fn dma_dealloc(
        _paddr: virtio_drivers::PhysAddr,
        vaddr: NonNull<u8>,
        pages: usize,
    ) -> i32 {
        let layout = Layout::array::<u8>(pages * PAGE_SIZE).unwrap();
        unsafe {
            HEAP.dealloc(vaddr.as_ptr(), layout);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: virtio_drivers::PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(
            to_virtual_address(
                PhysAddr::new(paddr),
                MEMORY_MAPPER
                    .try_get()
                    .expect("Heap not initialized")
                    .phys_offset()
                    .as_u64(),
            )
            .as_mut_ptr(),
        )
        .expect("Address is null")
    }

    unsafe fn share(
        buffer: NonNull<[u8]>,
        _direction: BufferDirection,
    ) -> virtio_drivers::PhysAddr {
        MEMORY_MAPPER
            .try_get()
            .expect("Heap not initialized")
            .translate_addr(VirtAddr::from_ptr(buffer.as_ptr()))
            .expect("Buffer is not in memory")
            .as_u64()
    }

    unsafe fn unshare(
        _paddr: virtio_drivers::PhysAddr,
        _buffer: NonNull<[u8]>,
        _direction: BufferDirection,
    ) {
    }
}
