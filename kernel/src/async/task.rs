use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};

use alloc::{boxed::Box, sync::Arc};
use crossbeam_queue::ArrayQueue;

use crate::r#async::waker::TaskWaker;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
    id: TaskId,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaskId {
    id: u64,
}

impl TaskId {
    pub fn new() -> TaskId {
        static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
        TaskId {
            id: TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Task {
    pub fn new(
        future: impl Future<Output = ()> + 'static,
        task_queue: Arc<ArrayQueue<TaskId>>,
    ) -> Self {
        let id = TaskId::new();
        Task {
            future: Box::pin(future),
            waker: TaskWaker::new(task_queue, id),
            id,
        }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn poll(&mut self) -> Poll<()> {
        let mut context = Context::from_waker(&self.waker);
        let poll = self.future.as_mut().poll(&mut context);
        poll
    }
}
