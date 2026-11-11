use crate::io::key_events;
use crate::tty::console_keyboard::ConsoleKeyboard;
use crate::tty::error_screen::ErrorScreen;
use crate::tty::input_buffer::InputBuffer;
use crate::tty::screen::Screen;
use alloc::sync::Arc;
use common::opened_resource::Resource;
use conquer_once::spin::Lazy;

mod console_keyboard;
mod error_screen;
mod input_buffer;
mod screen;

pub static TTY: Lazy<Tty> = Lazy::new(Tty::new);

pub fn init() {
    key_events::subscribe(Arc::new(ConsoleKeyboard::new()));
}

pub struct Tty {
    input: Arc<InputBuffer>,
    screen: Arc<Screen>,
    error_screen: Arc<ErrorScreen>,
}

impl Tty {
    pub fn new() -> Tty {
        Tty {
            input: Arc::new(InputBuffer::new()),
            screen: Arc::new(Screen),
            error_screen: Arc::new(ErrorScreen),
        }
    }

    pub fn receive(&self, bytes: &[u8]) {
        self.input.push(bytes);
    }

    pub fn stdin(&self) -> Arc<dyn Resource> {
        self.input.clone()
    }

    pub fn stdout(&self) -> Arc<dyn Resource> {
        self.screen.clone()
    }

    pub fn stderr(&self) -> Arc<dyn Resource> {
        self.error_screen.clone()
    }
}
