use conquer_once::spin::OnceCell;
use futures_util::StreamExt;
use pc_keyboard::{ScancodeSet, ScancodeSet1};

use super::stream::Streamer;
use crate::io::key_events;

pub static SCANCODE_STREAMER: OnceCell<Streamer<u8>> = OnceCell::uninit();

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

    let mut scancode_set = ScancodeSet1::new();

    while let Some(scancode) = stream.next().await {
        if let Ok(Some(event)) = scancode_set.advance_state(scancode) {
            key_events::publish(&event);
        }
    }
}
