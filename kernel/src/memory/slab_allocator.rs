use crate::memory::{FreeSpaceNode, PAGE_SIZE, binary_allocator::BinaryAllocator};
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

pub const MINIMUM_BLOCK_SIZE: u64 = size_of::<FreeSpaceNode>() as u64;
const SLAB_SIZE: u64 = PAGE_SIZE;
const MAX_DEPTH: u64 = (SLAB_SIZE / MINIMUM_BLOCK_SIZE).lowest_one().unwrap() as u64 + 1;

pub struct SlabAllocator<NewSlabAllocator: BinaryAllocator> {
    nodes: [Option<&'static mut FreeSpaceNode>; MAX_DEPTH as usize],
    new_slab_allocator: Option<NewSlabAllocator>,
}

impl<NewSlabAllocator: BinaryAllocator> SlabAllocator<NewSlabAllocator> {
    pub const fn new() -> SlabAllocator<NewSlabAllocator> {
        const EMPTY: Option<&'static mut FreeSpaceNode> = None;
        SlabAllocator {
            nodes: [EMPTY; MAX_DEPTH as usize],
            new_slab_allocator: None,
        }
    }

    pub fn init(&mut self, new_slab_allocator: NewSlabAllocator) {
        self.new_slab_allocator = Some(new_slab_allocator);
    }

    fn new_slab_allocator(&mut self) -> &mut NewSlabAllocator {
        self.new_slab_allocator
            .as_mut()
            .expect("init has not been called")
    }

    fn allocate_new_slab(&mut self, size: u64) -> Option<&'static mut FreeSpaceNode> {
        let ptr = self.new_slab_allocator().alloc(SLAB_SIZE)?;

        let mut root = None;
        let mut ptr = ptr as *mut FreeSpaceNode;
        for _ in 0..SLAB_SIZE / size {
            let node = unsafe {
                ptr.write(FreeSpaceNode::new());
                &mut *ptr
            };
            node.next = root.take();
            root = Some(node);
            ptr = unsafe { ptr.byte_add(size as usize) };
        }
        root
    }
}

impl<NewSlabAllocator: BinaryAllocator> BinaryAllocator for SlabAllocator<NewSlabAllocator> {
    fn alloc(&mut self, size: u64) -> Option<*mut u8> {
        if size >= PAGE_SIZE {
            return self.new_slab_allocator().alloc(size);
        }

        let depth = self.compute_depth(size)?;
        let slab = self.nodes[depth as usize]
            .take()
            .or_else(|| self.allocate_new_slab(size))?;
        self.nodes[depth as usize] = slab.next.take();
        Some(slab as *mut FreeSpaceNode as *mut u8)
    }

    fn dealloc(&mut self, ptr: *mut u8, size: u64) {
        if size >= PAGE_SIZE {
            return self.new_slab_allocator().dealloc(ptr, size);
        }

        let ptr = ptr as *mut FreeSpaceNode;
        let depth = self
            .compute_depth(size)
            .expect("Cannot deallocate more space than allocated");
        let node = unsafe {
            ptr.write(FreeSpaceNode::new());
            &mut *ptr
        };
        node.next = self.nodes[depth as usize].take();
        self.nodes[depth as usize] = Some(node);
    }

    fn minimum_block_size(&self) -> u64 {
        MINIMUM_BLOCK_SIZE
    }

    fn max_depth(&self) -> u64 {
        MAX_DEPTH
    }
}
