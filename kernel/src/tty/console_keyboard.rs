use crate::io::key_events::KeyEventHandler;
use crate::tty::TTY;
use pc_keyboard::{DecodedKey, EventDecoder, HandleControl, KeyCode, KeyEvent, layouts};
use spin::Mutex;

pub struct ConsoleKeyboard {
    decoder: Mutex<EventDecoder<layouts::Us104Key>>,
}

impl ConsoleKeyboard {
    pub const fn new() -> ConsoleKeyboard {
        ConsoleKeyboard {
            decoder: Mutex::new(EventDecoder::new(layouts::Us104Key, HandleControl::Ignore)),
        }
    }
}

impl KeyEventHandler for ConsoleKeyboard {
    fn handle(&self, event: &KeyEvent) {
        let key = self.decoder.lock().process_keyevent(event.clone());
        let Some(key) = key else {
            return;
        };

        let mut utf8 = [0u8; 4];
        let bytes: &[u8] = match key {
            DecodedKey::Unicode('\u{8}') => b"\x7f",
            DecodedKey::Unicode('\u{7f}') => b"\x1b[3~",
            DecodedKey::Unicode(character) => character.encode_utf8(&mut utf8).as_bytes(),
            DecodedKey::RawKey(KeyCode::ArrowUp) => b"\x1b[A",
            DecodedKey::RawKey(KeyCode::ArrowDown) => b"\x1b[B",
            DecodedKey::RawKey(KeyCode::ArrowRight) => b"\x1b[C",
            DecodedKey::RawKey(KeyCode::ArrowLeft) => b"\x1b[D",
            DecodedKey::RawKey(_) => return,
        };
        TTY.receive(bytes);
    }
}
