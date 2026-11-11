use alloc::sync::Arc;
use alloc::vec::Vec;
use common::keyboard::{KEY_EVENT_SIZE, KEY_EVENTS_PATH, KeyEvent};
use common::opened_resource::{BLOCK_SIZE, Resource};
use common::syscall_errors::read::ReadError;
use crossbeam_queue::ArrayQueue;
use spin::Mutex;

const QUEUE_CAPACITY: usize = BLOCK_SIZE / KEY_EVENT_SIZE;

/// Receives every key event published by the keyboard driver.
///
/// `handle` is called while the subscribers list is locked: it must never
/// call `subscribe` or `unsubscribe`.
pub trait KeyEventHandler: Send + Sync {
    fn handle(&self, event: &pc_keyboard::KeyEvent);
}

static SUBSCRIBERS: Mutex<Vec<Arc<dyn KeyEventHandler>>> = Mutex::new(Vec::new());

pub fn subscribe(handler: Arc<dyn KeyEventHandler>) {
    SUBSCRIBERS.lock().push(handler);
}

pub fn unsubscribe(handler: &Arc<dyn KeyEventHandler>) {
    SUBSCRIBERS
        .lock()
        .retain(|subscriber| !Arc::ptr_eq(subscriber, handler));
}

pub fn publish(event: &pc_keyboard::KeyEvent) {
    for subscriber in SUBSCRIBERS.lock().iter() {
        subscriber.handle(event);
    }
}

struct KeyEventQueue(ArrayQueue<KeyEvent>);

impl KeyEventHandler for KeyEventQueue {
    fn handle(&self, event: &pc_keyboard::KeyEvent) {
        if let Ok(event) = KeyEvent::try_from(event.clone()) {
            self.0.force_push(event);
        }
    }
}

pub struct KeyEvents {
    queue: Arc<KeyEventQueue>,
}

impl KeyEvents {
    pub fn open() -> KeyEvents {
        let queue = Arc::new(KeyEventQueue(ArrayQueue::new(QUEUE_CAPACITY)));
        subscribe(queue.clone());
        KeyEvents { queue }
    }
}

impl Resource for KeyEvents {
    fn read_into(&self, buffer: &mut [u8; BLOCK_SIZE]) -> Result<usize, ReadError> {
        let mut size = 0;
        for chunk in buffer.chunks_exact_mut(KEY_EVENT_SIZE) {
            let Some(event) = self.queue.0.pop() else {
                break;
            };
            chunk.copy_from_slice(&event.to_bytes());
            size += KEY_EVENT_SIZE;
        }
        Ok(size)
    }

    fn write(&self, _data: &[u8]) -> Result<(), ()> {
        Err(())
    }

    fn name(&self) -> &str {
        KEY_EVENTS_PATH
    }
}

impl Drop for KeyEvents {
    fn drop(&mut self) {
        // The clone is just an Arc::clone
        let handler: Arc<dyn KeyEventHandler> = self.queue.clone();
        unsubscribe(&handler);
    }
}
