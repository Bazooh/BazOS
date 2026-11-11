use crate::tty::error_screen::ErrorScreen;
use crate::tty::input_buffer::InputBuffer;
use crate::tty::screen::Screen;
use crate::{erase, print};
use pc_keyboard::DecodedKey;

mod error_screen;
mod input_buffer;
mod screen;

pub static TTY: Tty = Tty::new();

pub struct Tty {
    input: InputBuffer,
    screen: Screen,
    error_screen: ErrorScreen,
}

impl Tty {
    pub const fn new() -> Tty {
        Tty {
            input: InputBuffer::new(),
            screen: Screen,
            error_screen: ErrorScreen,
        }
    }

    pub fn handle(&self, key: DecodedKey) {
        self.input.handle(key);
        match key {
            DecodedKey::Unicode('\u{8}') => erase!(),
            DecodedKey::Unicode(character) => print!("{}", character),
            DecodedKey::RawKey(_key) => (),
        }
    }
}
