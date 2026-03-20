use core::{alloc::Layout, ptr::NonNull};

use crate::{
    memory::heap::{HEAP_SIZE, HEAP_START},
    println,
};

#[repr(C, align(16))]
struct FreeSpaceNode {
    next: Option<&'static mut FreeSpaceNode>,
}

impl FreeSpaceNode {
    fn new() -> FreeSpaceNode {
        FreeSpaceNode { next: None }
    }
}

const MINIMUM_BLOCK_SIZE: usize = 16;
const DEPTH: usize = (HEAP_SIZE / MINIMUM_BLOCK_SIZE).lowest_one().unwrap() as usize + 1;
const DEPTH_OFFSET: usize = HEAP_SIZE.lowest_one().unwrap() as usize;

pub struct BuddyAllocator {
    nodes: [Option<&'static mut FreeSpaceNode>; DEPTH as usize],
}

impl BuddyAllocator {
    pub const fn new() -> BuddyAllocator {
        const EMPTY: Option<&'static mut FreeSpaceNode> = None;
        BuddyAllocator {
            nodes: [EMPTY; DEPTH as usize],
        }
    }

    pub fn init(&mut self) {
        assert!(HEAP_SIZE.is_power_of_two());
        let ptr = HEAP_START as *mut FreeSpaceNode;
        unsafe {
            ptr.write(FreeSpaceNode::new());
            self.nodes[0] = Some(&mut *ptr);
        }
    }

    fn compute_depth(&self, block_size: usize) -> Option<usize> {
        let highest_one = (block_size - 1).highest_one().map(|x| x as usize);
        match highest_one {
            Some(highest_one) => {
                const MINIMUM_HIGH_ONE: usize =
                    (MINIMUM_BLOCK_SIZE - 1).highest_one().unwrap() as usize;
                if highest_one <= MINIMUM_HIGH_ONE {
                    Some(DEPTH - 1)
                } else if highest_one > DEPTH + MINIMUM_HIGH_ONE - 1 {
                    None
                } else {
                    Some(DEPTH + MINIMUM_HIGH_ONE - 1 - highest_one)
                }
            }
            None => Some(DEPTH - 1),
        }
    }

    fn compute_right_buddy_ptr(
        &self,
        depth: usize,
        ptr: *const FreeSpaceNode,
    ) -> Option<*mut FreeSpaceNode> {
        if depth == 0 {
            return None;
        }

        unsafe { Some((ptr as *mut u8).add(1 << (DEPTH_OFFSET - depth)) as *mut FreeSpaceNode) }
    }

    fn compute_left_buddy_ptr(
        &self,
        depth: usize,
        ptr: *const FreeSpaceNode,
    ) -> Option<*mut FreeSpaceNode> {
        if depth == 0 {
            return None;
        }

        unsafe { Some((ptr as *mut u8).sub(1 << (DEPTH_OFFSET - depth)) as *mut FreeSpaceNode) }
    }

    fn compute_buddy_ptr(
        &self,
        depth: usize,
        ptr: *const FreeSpaceNode,
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
        (ptr as usize - HEAP_START) & (1 << (DEPTH_OFFSET - depth)) == 0
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

            if (buddy_ptr as usize) < HEAP_START {
                panic!("take or divide: ptr: {:?}, depth: {}", buddy_ptr, depth);
            }
            unsafe {
                buddy_ptr.write(FreeSpaceNode::new());
                &mut *buddy_ptr
            }
        })
    }

    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let size = layout.size().max(layout.align());
        let depth = self.compute_depth(size)?;
        Some(self.take_or_divide(depth)? as *mut FreeSpaceNode as *mut u8)
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
            if (ptr as usize) < HEAP_START {
                panic!("merge: ptr: {:?}, depth: {}", ptr, depth);
            }
            unsafe {
                ptr.write(FreeSpaceNode::new());
                self.nodes[0] = Some(&mut *ptr);
            }
            return;
        }

        let is_left = self.is_left(depth, ptr);
        let buddy_ptr = self
            .compute_buddy_ptr(depth, ptr as *const FreeSpaceNode, is_left)
            .expect("depth should be > 0");
        let buddy = self.remove_ptr_from_list(depth, buddy_ptr);

        match buddy {
            Some(_) => self.merge(depth - 1, if is_left { ptr } else { buddy_ptr }),
            None => {
                let mut node = FreeSpaceNode::new();
                node.next = self.nodes[depth].take();
                if (ptr as usize) < HEAP_START {
                    panic!("merge: ptr: {:?}, depth: {}", ptr, depth);
                }
                unsafe {
                    ptr.write(node);
                    self.nodes[depth] = Some(&mut *ptr);
                }
            }
        }
    }

    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if (ptr as usize) < HEAP_START {
            panic!("dealloc: ptr: {:?}, size: {:?}", ptr, layout);
        }

        let size = layout.size().max(layout.align());
        let depth = self.compute_depth(size);
        if depth.is_none() {
            panic!("dealloc is none: ptr: {:?}, size: {}", ptr, size);
        }
        self.merge(depth.unwrap(), ptr as *mut FreeSpaceNode);
    }
}
