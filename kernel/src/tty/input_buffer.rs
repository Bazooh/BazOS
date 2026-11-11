use common::opened_resource::{BLOCK_SIZE, Resource};
use common::syscall_errors::read::ReadError;
use crossbeam_queue::ArrayQueue;

pub struct InputBuffer {
    buffer: ArrayQueue<u8>,
}

impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            buffer: ArrayQueue::new(BLOCK_SIZE),
        }
    }

    pub fn push(&self, bytes: &[u8]) {
        for &byte in bytes {
            self.buffer.force_push(byte);
        }
    }
}

impl Resource for InputBuffer {
    fn read_into(&self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        for i in 0..BLOCK_SIZE {
            let Some(byte) = self.buffer.pop() else {
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
