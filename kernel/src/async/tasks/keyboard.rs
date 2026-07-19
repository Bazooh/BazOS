use conquer_once::spin::OnceCell;
use futures_util::StreamExt;
use pc_keyboard::{DecodedKey, Keyboard, KeyboardLayout, ScancodeSet, ScancodeSet1, layouts};

use crate::{erase, print};

use super::stream::Streamer;

pub static SCANCODE_STREAMER: OnceCell<Streamer<u8>> = OnceCell::uninit();

trait KeyProcessor {
    fn process(&mut self, key_code: u8);
}

impl<L: KeyboardLayout, S: ScancodeSet> KeyProcessor for Keyboard<L, S> {
    fn process(&mut self, scancode: u8) {
        let key = {
            if let Ok(Some(key_event)) = self.add_byte(scancode) {
                self.process_keyevent(key_event)
            } else {
                None
            }
        };

        if let Some(key) = key {
            match key {
                DecodedKey::Unicode('\u{8}') => erase!(),
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(_key) => (),
            }
        }
    }
}

pub fn init_keyboard_streamer() {
    SCANCODE_STREAMER
        .try_init_once(|| Streamer::new(64))
        .expect("Streamer already init");
}

pub async fn handle_key_presses() {
    let mut stream = SCANCODE_STREAMER
        .try_get()
        .expect("Streamer uninit")
        .stream();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        pc_keyboard::HandleControl::Ignore,
    );

    while let Some(scancode) = stream.next().await {
        keyboard.process(scancode);
    }
}
