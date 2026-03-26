use crate::r#async::tasks::keyboard::init_keyboard_streamer;

pub mod executor;
mod process;
mod scheduler;
mod task;
mod tasks;
mod thread;
mod waker;

pub use tasks::keyboard::SCANCODE_STREAMER;

pub fn init_tasks() {
    init_keyboard_streamer();
}
