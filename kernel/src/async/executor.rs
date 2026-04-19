use core::task::Poll;

use alloc::{collections::BTreeMap, sync::Arc};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::hlt;

use crate::println;

use super::{
    task::{Task, TaskId},
    tasks::keyboard::handle_keyboard_interrupt,
};

pub struct Executor {
    task_queue: Arc<ArrayQueue<TaskId>>,
    tasks: BTreeMap<TaskId, Task>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            task_queue: Arc::new(ArrayQueue::new(128)),
            tasks: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let task = Task::new(future, Arc::clone(&self.task_queue));
        let task_id = task.id();

        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with same ID already in tasks");
        }

        if let Err(_) = self.task_queue.push(task_id) {
            println!("WARNING: task queue full; dropping task");
            self.tasks.remove(&task_id);
        }
    }

    pub fn run(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            match self.tasks.get_mut(&task_id).expect("Task not found").poll() {
                Poll::Ready(()) => {
                    self.tasks.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn kernel() -> ! {
        let mut executor = Executor::new();
        executor.spawn(handle_keyboard_interrupt());
        loop {
            executor.run();
            hlt();
        }
    }
}
