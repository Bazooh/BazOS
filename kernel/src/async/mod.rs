use crate::r#async::{scheduler::init_scheduler, tasks::keyboard::init_keyboard_streamer};

pub mod executor;
pub mod process;
pub mod scheduler;
mod task;
mod tasks;
pub mod thread;
mod waker;

pub use tasks::keyboard::SCANCODE_STREAMER;

pub fn init_async() {
    init_scheduler();
    init_keyboard_streamer();
}
