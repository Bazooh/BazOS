use common::opened_resource::{Resource, BLOCK_SIZE};
use common::syscall_errors::read::ReadError;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::{DecodedKey, KeyCode};

pub struct InputBuffer {
    buffer: OnceCell<ArrayQueue<u8>>,
}

impl InputBuffer {
    pub const fn new() -> InputBuffer {
        InputBuffer {
            buffer: OnceCell::uninit(),
        }
    }

    pub fn handle(&self, key: DecodedKey) {
        let mut utf8 = [0u8; 4];
        let bytes: &[u8] = match key {
            DecodedKey::Unicode(character) => character.encode_utf8(&mut utf8).as_bytes(),
            DecodedKey::RawKey(KeyCode::ArrowUp) => b"\x1b[A",
            DecodedKey::RawKey(KeyCode::ArrowDown) => b"\x1b[B",
            DecodedKey::RawKey(KeyCode::ArrowRight) => b"\x1b[C",
            DecodedKey::RawKey(KeyCode::ArrowLeft) => b"\x1b[D",
            DecodedKey::RawKey(_) => return,
        };

        let queue = self
            .buffer
            .get_or_init(|| ArrayQueue::new(BLOCK_SIZE));
        for &byte in bytes {
            queue.force_push(byte);
        }
    }
}

impl Resource for InputBuffer {
    fn read_into(&mut self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
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

    fn write(&mut self, _data: &[u8]) -> Result<(), ()> {
        Err(())
    }

    fn name(&self) -> &str {
        "keyboard.x"
    }
}
