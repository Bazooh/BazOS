use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, KeyCode, Keyboard, ScancodeSet1, ScancodeSet2, layouts};
use spin::Mutex;
use x86_64::instructions::port::Port;

use crate::{
    erase,
    interrupts::{
        hardware::{HardwareInterrupt, PICS},
        idt::ExceptionStackFrame,
    },
    print,
};

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            pc_keyboard::HandleControl::Ignore
        ));
}

pub extern "C" fn keyboard_handler(_stack_frame: &ExceptionStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

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

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(HardwareInterrupt::Keyboard as u8);
    }
}
