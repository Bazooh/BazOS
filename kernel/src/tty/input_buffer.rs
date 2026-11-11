use common::opened_resource::{BLOCK_SIZE, Resource};
use common::syscall_errors::read::ReadError;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;

pub struct InputBuffer {
    buffer: OnceCell<ArrayQueue<u8>>,
}

impl InputBuffer {
    pub const fn new() -> InputBuffer {
        InputBuffer {
            buffer: OnceCell::uninit(),
        }
    }

    pub fn push(&self, bytes: &[u8]) {
        let queue = self.buffer.get_or_init(|| ArrayQueue::new(BLOCK_SIZE));
        for &byte in bytes {
            queue.force_push(byte);
        }
    }
}

impl Resource for InputBuffer {
    fn read_into(&self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        let Some(queue) = self.buffer.get() else {
            return Ok(0);
        };

        for i in 0..BLOCK_SIZE {
            let Some(byte) = queue.pop() else {
                return Ok(i);
            };
            buffer[i] = byte;
        }
        Ok(BLOCK_SIZE)
    }

    fn write(&self, _data: &[u8]) -> Result<(), ()> {
        Err(())
    }

    fn name(&self) -> &str {
        "keyboard.x"
    }
}
