use core::alloc::Layout;

use spin::Mutex;

use crate::memory::{FreeSpaceNode, PAGE_SIZE, binary_allocator::BinaryAllocator};

const MINIMUM_BLOCK_SIZE: usize = size_of::<FreeSpaceNode>();
const SLAB_SIZE: usize = PAGE_SIZE;
const MAX_DEPTH: usize = (SLAB_SIZE / MINIMUM_BLOCK_SIZE).lowest_one().unwrap() as usize + 1;

pub struct SlabAllocator<NewSlabAllocator: BinaryAllocator> {
    nodes: [Option<&'static mut FreeSpaceNode>; MAX_DEPTH as usize],
    new_slab_allocator: Option<Mutex<NewSlabAllocator>>,
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
        self.new_slab_allocator = Some(Mutex::new(new_slab_allocator));
    }

    fn new_slab_allocator(&mut self) -> &mut Mutex<NewSlabAllocator> {
        self.new_slab_allocator
            .as_mut()
            .expect("init has not been called")
    }

    fn allocate_new_slab(&mut self, size: usize) -> Option<&'static mut FreeSpaceNode> {
        let ptr = self
            .new_slab_allocator()
            .lock()
            .alloc(Layout::from_size_align(SLAB_SIZE, SLAB_SIZE).expect("Alignement is wrong"))?;

        let mut root = None;
        let mut ptr = ptr as *mut FreeSpaceNode;
        for _ in 0..SLAB_SIZE / size {
            let node = unsafe {
                ptr.write(FreeSpaceNode::new());
                &mut *ptr
            };
            node.next = root.take();
            root = Some(node);
            ptr = unsafe { ptr.byte_add(size) };
        }
        root
    }
}

impl<NewSlabAllocator: BinaryAllocator> BinaryAllocator for SlabAllocator<NewSlabAllocator> {
    fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let size = layout.size().max(layout.align());

        if size >= PAGE_SIZE {
            return self.new_slab_allocator().lock().alloc(layout);
        }

        let depth = self.compute_depth(size)?;
        let slab = self.nodes[depth]
            .take()
            .or_else(|| self.allocate_new_slab(size))?;
        self.nodes[depth] = slab.next.take();
        Some(slab as *mut FreeSpaceNode as *mut u8)
    }

    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(layout.align());

        if size >= PAGE_SIZE {
            return self.new_slab_allocator().lock().dealloc(ptr, layout);
        }

        let ptr = ptr as *mut FreeSpaceNode;
        let depth = self
            .compute_depth(size)
            .expect("Cannot deallocate more space than allocated");
        let node = unsafe {
            ptr.write(FreeSpaceNode::new());
            &mut *ptr
        };
        node.next = self.nodes[depth].take();
        self.nodes[depth] = Some(node);
    }

    fn minimum_block_size(&self) -> usize {
        MINIMUM_BLOCK_SIZE
    }

    fn max_depth(&self) -> usize {
        MAX_DEPTH
    }
}
