use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crossbeam_queue::ArrayQueue;
use futures_util::{Stream, task::AtomicWaker};

use crate::println;

pub struct Streamer<Item> {
    queue: ArrayQueue<Item>,
    waker: AtomicWaker,
}

impl<Item> Streamer<Item> {
    pub fn new(capacity: usize) -> Self {
        Streamer {
            queue: ArrayQueue::new(capacity),
            waker: AtomicWaker::new(),
        }
    }

    pub fn push(&self, item: Item) {
        if let Err(_) = self.queue.push(item) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            self.waker.wake();
        }
    }

    pub fn stream(&'static self) -> impl Stream<Item = Item> {
        TaskStream::new(self)
    }
}

struct TaskStream<Item: 'static> {
    streamer: &'static Streamer<Item>,
}

impl<Item> TaskStream<Item> {
    pub fn new(streamer: &'static Streamer<Item>) -> Self {
        TaskStream { streamer }
    }
}

impl<Item> Stream for TaskStream<Item> {
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Item>> {
        let queue = &self.streamer.queue;

        // fast path
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        let waker = &self.streamer.waker;

        waker.register(cx.waker());
        match queue.pop() {
            Some(scancode) => {
                waker.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}
