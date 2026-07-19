use crate::{interrupts::idt::ExceptionStackFrame, println};
use bit_field::BitField;
use core::fmt::Display;

struct GeneralFaultErrorCode(u32);

impl GeneralFaultErrorCode {
    fn from_u64(value: u64) -> GeneralFaultErrorCode {
        GeneralFaultErrorCode(value as u32)
    }
}

impl Display for GeneralFaultErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let external = self.0.get_bit(0);
        let table = self.0.get_bit(1);
        let index = self.0 >> 3;

        write!(f, "Selector index: {}, ", index)?;

        if table {
            write!(f, "LDT, ")?;
        } else {
            write!(f, "GDT, ")?;
        }

        if external {
            write!(f, "EXTERNAL")?;
        } else {
            write!(f, "INTERNAL")?;
        }

        Ok(())
    }
}

pub extern "C" fn general_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    println!(
        "EXCEPTION: GENERAL FAULT\n  with error code {}\n{:#?}\n",
        GeneralFaultErrorCode::from_u64(error_code),
        stack_frame
    );
    loop {}
}
