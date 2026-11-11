use alloc::sync::Arc;
use alloc::vec::Vec;
use common::keyboard::{KeyEvent, KEY_EVENTS_PATH, KEY_EVENT_SIZE};
use common::opened_resource::{Resource, BLOCK_SIZE};
use common::syscall_errors::read::ReadError;
use crossbeam_queue::ArrayQueue;
use spin::Mutex;

const QUEUE_CAPACITY: usize = BLOCK_SIZE / KEY_EVENT_SIZE;

static SUBSCRIBERS: Mutex<Vec<Arc<ArrayQueue<KeyEvent>>>> = Mutex::new(Vec::new());

pub fn publish(event: pc_keyboard::KeyEvent) {
    let Ok(event) = KeyEvent::try_from(event) else {
        return;
    };

    for subscriber in SUBSCRIBERS.lock().iter() {
        subscriber.force_push(event);
    }
}

pub struct KeyEvents {
    queue: Arc<ArrayQueue<KeyEvent>>,
}

impl KeyEvents {
    pub fn open() -> KeyEvents {
        let queue = Arc::new(ArrayQueue::new(QUEUE_CAPACITY));
        SUBSCRIBERS.lock().push(Arc::clone(&queue));
        KeyEvents { queue }
    }
}

impl Resource for KeyEvents {
    fn read_into(&mut self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        let mut size = 0;
        for chunk in buffer.chunks_exact_mut(KEY_EVENT_SIZE) {
            let Some(event) = self.queue.pop() else {
                break;
            };
            chunk.copy_from_slice(&event.to_bytes());
            size += KEY_EVENT_SIZE;
        }
        Ok(size)
    }

    fn write(&mut self, _data: &[u8]) -> Result<(), ()> {
        Err(())
    }

    fn name(&self) -> &str {
        KEY_EVENTS_PATH
    }
}

impl Drop for KeyEvents {
    fn drop(&mut self) {
        SUBSCRIBERS
            .lock()
            .retain(|subscriber| !Arc::ptr_eq(subscriber, &self.queue));
    }
}
