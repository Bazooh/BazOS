use core::ptr::null_mut;

use crate::memory::{
    FreeSpaceNode, HEAP_SIZE, HEAP_START, PAGE_SIZE,
    binary_allocator::BinaryAllocator,
    program_allocator::{PROGRAM_SIZE, PROGRAM_START},
};

pub struct BuddyAllocator<const MAX_DEPTH: usize> {
    nodes: [Option<&'static mut FreeSpaceNode>; MAX_DEPTH],
    minimum_block_size: usize,
    depth_offset: usize,
    mem_start: usize,
}

pub const fn compute_max_depth(size: usize, minimum_block_size: usize) -> usize {
    (size / minimum_block_size).lowest_one().unwrap() as usize + 1
}

impl<const MAX_DEPTH: usize> BuddyAllocator<MAX_DEPTH> {
    pub const fn new(size: usize, minimum_block_size: usize, mem_start: usize) -> Self {
        assert!(size.is_power_of_two());
        assert!(MAX_DEPTH == compute_max_depth(size, minimum_block_size));

        const EMPTY: Option<&'static mut FreeSpaceNode> = None;
        BuddyAllocator {
            nodes: [EMPTY; MAX_DEPTH],
            minimum_block_size,
            depth_offset: size.lowest_one().unwrap() as usize,
            mem_start,
        }
    }

    pub fn init(&mut self) {
        let ptr = self.mem_start as *mut FreeSpaceNode;
        unsafe {
            ptr.write(FreeSpaceNode::new());
            self.nodes[0] = Some(&mut *ptr);
        }
    }

    fn compute_right_buddy_ptr(
        &self,
        depth: usize,
        ptr: *mut FreeSpaceNode,
    ) -> Option<*mut FreeSpaceNode> {
        if depth == 0 {
            return None;
        }

        unsafe { Some(ptr.byte_add(1 << (self.depth_offset - depth))) }
    }

    fn compute_left_buddy_ptr(
        &self,
        depth: usize,
        ptr: *mut FreeSpaceNode,
    ) -> Option<*mut FreeSpaceNode> {
        if depth == 0 {
            return None;
        }

        unsafe { Some(ptr.byte_sub(1 << (self.depth_offset - depth))) }
    }

    fn compute_buddy_ptr(
        &self,
        depth: usize,
        ptr: *mut FreeSpaceNode,
        is_left: bool,
    ) -> Option<*mut FreeSpaceNode> {
        if depth == 0 {
            return None;
        }

        if is_left {
            self.compute_right_buddy_ptr(depth, ptr)
        } else {
            self.compute_left_buddy_ptr(depth, ptr)
        }
    }

    fn is_left(&self, depth: usize, ptr: *const FreeSpaceNode) -> bool {
        (ptr as usize - self.mem_start) & (1 << (self.depth_offset - depth)) == 0
    }

    fn take_or_divide(&mut self, depth: usize) -> Option<&'static mut FreeSpaceNode> {
        if let Some(node) = self.nodes[depth].take() {
            self.nodes[depth] = node.next.take();
            return Some(node);
        }

        if depth == 0 {
            return None;
        }

        self.take_or_divide(depth - 1).map(|free_space_node| {
            let buddy_ptr = self
                .compute_right_buddy_ptr(depth, free_space_node)
                .expect("depth should be > 0");

            free_space_node.next = self.nodes[depth].take();
            self.nodes[depth] = Some(free_space_node);
            unsafe {
                buddy_ptr.write(FreeSpaceNode::new());
                &mut *buddy_ptr
            }
        })
    }

    fn remove_ptr_from_list(
        &mut self,
        depth: usize,
        ptr: *mut FreeSpaceNode,
    ) -> Option<&'static mut FreeSpaceNode> {
        if self.nodes[depth].is_none() {
            return None;
        }

        let mut previous = self.nodes[depth].as_mut().unwrap();
        while let Some(ref mut node) = previous.next {
            if ptr == *node as *mut FreeSpaceNode {
                let next = node.next.take();
                let current = previous.next.take();
                previous.next = next;
                return current;
            }
            previous = previous.next.as_mut().unwrap();
        }
        None
    }

    fn merge(&mut self, depth: usize, ptr: *mut FreeSpaceNode) {
        if depth == 0 {
            unsafe {
                ptr.write(FreeSpaceNode::new());
                self.nodes[0] = Some(&mut *ptr);
            }
            return;
        }

        let is_left = self.is_left(depth, ptr);
        let buddy_ptr = self
            .compute_buddy_ptr(depth, ptr, is_left)
            .expect("depth should be > 0");
        let buddy = self.remove_ptr_from_list(depth, buddy_ptr);

        match buddy {
            Some(_) => self.merge(depth - 1, if is_left { ptr } else { buddy_ptr }),
            None => {
                let mut node = FreeSpaceNode::new();
                node.next = self.nodes[depth].take();
                unsafe {
                    ptr.write(node);
                    self.nodes[depth] = Some(&mut *ptr);
                }
            }
        }
    }
}

impl<const MAX_DEPTH: usize> BinaryAllocator for BuddyAllocator<MAX_DEPTH> {
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let depth = self.compute_depth(size)?;
        Some(self.take_or_divide(depth)? as *mut FreeSpaceNode as *mut u8)
    }

    fn dealloc(&mut self, ptr: *mut u8, size: usize) {
        let depth = self.compute_depth(size).unwrap();
        self.merge(depth, ptr as *mut FreeSpaceNode);
    }

    fn minimum_block_size(&self) -> usize {
        self.minimum_block_size
    }

    fn max_depth(&self) -> usize {
        MAX_DEPTH
    }
}
