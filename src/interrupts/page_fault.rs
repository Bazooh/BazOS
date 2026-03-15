use core::fmt::Display;

use bit_field::BitField;
use x86_64::{instructions::interrupts::int3, registers::control};

use crate::{interrupts::idt::ExceptionStackFrame, println};

struct PageFaultErrorCode(u8);

impl PageFaultErrorCode {
    fn from_u64(value: u64) -> Option<PageFaultErrorCode> {
        if value & !0b11111 != 0 {
            return None;
        }
        Some(PageFaultErrorCode(value as u8))
    }
}

impl Display for PageFaultErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        for i in 0..5 {
            if self.0.get_bit(i) {
                if !first {
                    write!(f, ", ")?;
                }
                first = false;
                match i {
                    0 => write!(f, "PROTECTION_VIOLATION")?,
                    1 => write!(f, "CAUSED_BY_WRITE")?,
                    2 => write!(f, "USER_MODE")?,
                    3 => write!(f, "MALFORMED_TABLE")?,
                    4 => write!(f, "INSTRUCTION_FETCH")?,
                    _ => unreachable!(),
                }
            }
        }
        Ok(())
    }
}

pub extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    println!(
        "EXCEPTION: PAGE FAULT\n  while trying to access address {:#x}\n  with error code {}\n{:#?}",
        control::Cr2::read().unwrap(),
        PageFaultErrorCode::from_u64(error_code).unwrap(),
        stack_frame
    );
    loop {}
}
