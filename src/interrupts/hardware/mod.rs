use pic8259::ChainedPics;
use spin::Mutex;

pub mod timer;
pub mod keyboard;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[repr(u8)]
pub enum HardwareInterrupt {
    Timer = PIC_1_OFFSET,
    Keyboard,
}
