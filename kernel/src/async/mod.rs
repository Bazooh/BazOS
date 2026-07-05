use crate::r#async::tasks::keyboard::init_keyboard_streamer;

pub mod executor;
pub mod process;
pub mod scheduling;
mod task;
mod tasks;
pub mod thread;
mod threads_array;
mod waker;

use crate::r#async::scheduling::scheduler::init_scheduler;
pub use tasks::keyboard::SCANCODE_STREAMER;

pub fn init_async() {
    init_scheduler();
    init_keyboard_streamer();
}
