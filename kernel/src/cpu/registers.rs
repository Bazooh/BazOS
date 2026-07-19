#![allow(unused)]

use core::arch::asm;

pub trait Register {
    fn read(&self) -> u64;
    fn write(&mut self, value: u64);
}

macro_rules! register {
    ($reg:ident) => {
        paste::paste! {
            struct [< $reg:camel >];

            impl Register for [< $reg:camel >] {
                fn read(&self) -> u64 {
                    let mut value: u64;
                    unsafe {
                        core::arch::asm!(
                            concat!("mov {}, ", stringify!($reg)),
                            out(reg) value
                        );
                    }
                    value
                }

                fn write(&mut self, value: u64) {
                    unsafe {
                        core::arch::asm!(
                            concat!("mov ", stringify!($reg), ", {}"),
                            in(reg) value
                        );
                    }
                }
            }

            pub fn $reg() -> impl Register {
                [< $reg:camel >]
            }
        }
    };
}

register!(cr3);
register!(rsp);
