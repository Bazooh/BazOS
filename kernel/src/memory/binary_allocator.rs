/// An allocator that can only allocate memory sizes that are a power of 2
pub trait BinaryAllocator {
    fn alloc(&mut self, size: usize) -> Option<*mut u8>;

    fn dealloc(&mut self, ptr: *mut u8, size: usize);

    fn minimum_block_size(&self) -> usize;

    fn max_depth(&self) -> usize;

    fn compute_depth(&self, block_size: usize) -> Option<usize> {
        // TODO: Change this algorithm to take into account that `block_size` is a power of 2

        let highest_one = (block_size - 1).highest_one().map(|x| x as usize);
        let max_depth_inclusive = self.max_depth() - 1;
        match highest_one {
            Some(highest_one) => {
                let minimum_highest_one: usize =
                    (self.minimum_block_size() - 1).highest_one().unwrap() as usize;
                if highest_one <= minimum_highest_one {
                    // block_size is too small give the minimum size
                    return Some(max_depth_inclusive);
                };
                if max_depth_inclusive + minimum_highest_one < highest_one {
                    // block_size is too big => allocation failed
                    None
                } else {
                    Some(max_depth_inclusive + minimum_highest_one - highest_one)
                }
            }
            None => Some(max_depth_inclusive),
        }
    }
}
