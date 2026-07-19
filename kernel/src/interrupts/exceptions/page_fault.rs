use crate::{interrupts::idt::ExceptionStackFrame, println};
use bit_field::BitField;
use core::{arch::asm, fmt::Display};
use x86_64::registers::control;

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
        for i in 0..5 {
            let set = self.0.get_bit(i);
            match (i, set) {
                (0, true) => {}
                (0, false) => write!(f, "NOT MAPPED, ")?,
                (1, true) => write!(f, "WRITE, ")?,
                (1, false) => write!(f, "READ, ")?,
                (2, true) => write!(f, "USER, ")?,
                (2, false) => write!(f, "KERNEL, ")?,
                (3, true) => write!(f, "MALFORMED TABLE")?,
                (3, false) => {}
                (4, true) => write!(f, "INSTRUCTION FETCH")?,
                (4, false) => {}
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

pub extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    let r15: usize;
    unsafe {
        asm!("mov {}, r15", out(reg) r15);
    }
    println!(
        "EXCEPTION: PAGE FAULT\n  while trying to access address VirtAddr({:#x})\n  with error code {}\n{:#?}\nr15: {r15:x}",
        control::Cr2::read().unwrap(),
        PageFaultErrorCode::from_u64(error_code).unwrap(),
        stack_frame
    );
    loop {}
}
