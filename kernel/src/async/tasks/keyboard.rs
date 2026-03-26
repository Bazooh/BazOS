use conquer_once::spin::OnceCell;
use futures_util::StreamExt;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1, layouts};
use spin::Mutex;

use crate::{r#async::tasks::stream::Streamer, erase, print};

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            pc_keyboard::HandleControl::Ignore
        ));
}

pub static SCANCODE_STREAMER: OnceCell<Streamer<u8>> = OnceCell::uninit();

fn process(scancode: u8) {
    let key = {
        let mut keyboard = KEYBOARD.lock();
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            keyboard.process_keyevent(key_event)
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

pub fn init_keyboard_streamer() {
    SCANCODE_STREAMER
        .try_init_once(|| Streamer::new(64))
        .expect("Streamer already init");
}

pub async fn handle_keyboard_interrupt() {
    let mut stream = SCANCODE_STREAMER
        .try_get()
        .expect("Streamer uninit")
        .stream();

    while let Some(scancode) = stream.next().await {
        process(scancode);
    }
}
