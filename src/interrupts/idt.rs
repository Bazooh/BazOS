use core::{
    arch::{asm, naked_asm},
    fmt::{Debug, Display, Formatter, LowerHex},
    ops::{Deref, DerefMut},
    ptr::{addr_of_mut, read_unaligned},
};

use bit_field::BitField;
use lazy_static::lazy_static;
use x86_64::{
    VirtAddr,
    instructions::{segmentation, tables::lidt},
    registers::{control, segmentation::Segment},
    structures::{DescriptorTablePointer, gdt::SegmentSelector},
};

use crate::{
    interrupts::{
        breakpoint::breakpoint_handler,
        divide_by_zero::divide_by_zero_handler,
        double_fault::double_fault_handler,
        hardware::{HardwareInterrupt, PICS, keyboard::keyboard_handler, timer::timer_handler},
        invalid_opcode::invalid_opcode_handler,
        page_fault::page_fault_handler,
    },
    println,
    utils::debug::DebugHex,
};

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Entry {
    pub fn new(gdt_selector: SegmentSelector, handler: FunctionHandler) -> Entry {
        let pointer = handler as u64;
        Entry {
            pointer_low: pointer as u16,
            gdt_selector,
            options: EntryOptions::new(),
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            reserved: 0,
        }
    }

    pub fn missing() -> Entry {
        Entry {
            pointer_low: 0,
            gdt_selector: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring0),
            options: EntryOptions::minimal(),
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EntryOptions(u16);

impl EntryOptions {
    pub fn minimal() -> EntryOptions {
        let mut value = 0;
        value.set_bits(9..=11, 0b111);
        EntryOptions(value)
    }

    pub fn new() -> EntryOptions {
        let mut options = EntryOptions::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut EntryOptions {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut EntryOptions {
        self.0.set_bit(8, !disable);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut EntryOptions {
        self.0.set_bits(13..=14, dpl);
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut EntryOptions {
        self.0.set_bits(0..=2, index + 1);
        self
    }
}

type FunctionHandler = extern "C" fn() -> !;

struct InteruptDescriptorTable([Entry; 64]);

impl InteruptDescriptorTable {
    pub fn new() -> InteruptDescriptorTable {
        InteruptDescriptorTable([Entry::missing(); 64])
    }

    pub fn set_handler(&mut self, entry: u8, handler: FunctionHandler) -> &mut EntryOptions {
        self.0[entry as usize] = Entry::new(segmentation::CS::get_reg(), handler);
        let address = &raw mut self.0[entry as usize].options;
        // This is safe because the value is guaranteed to be 16-aligned
        // because all fields in Entry are 16-aligned
        unsafe { &mut *address }
    }

    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: VirtAddr::from_ptr(self),
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}

macro_rules! save_scratch_reg {
    () => {
        "push rax
         push rcx
         push rdx
         push rsi
         push rdi
         push r8
         push r9
         push r10
         push r11"
    };
}

macro_rules! restore_scratch_reg {
    () => {
        "pop r11
         pop r10
         pop r9
         pop r8
         pop rsi
         pop rdi
         pop rdx
         pop rcx
         pop rax"
    };
}

type ExceptionHandler = extern "C" fn(&ExceptionStackFrame);

macro_rules! handler {
    ($name: ident) => {{
        // Ensure type safety
        $name as ExceptionHandler;

        #[unsafe(naked)]
        extern "C" fn wrapper() -> ! {
            naked_asm!(
                save_scratch_reg!(),
                "mov rdi, rsp
                 add rdi, 8*9
                 call {func}",
                restore_scratch_reg!(),
                "iretq",
                func = sym $name
            );
        }
        wrapper
    }};
}

type ErrorCodeHandler = extern "C" fn(&ExceptionStackFrame, u64);

macro_rules! handler_with_error_code {
    ($name: ident) => {{
        // Ensure type safety
        $name as ErrorCodeHandler;

        #[unsafe(naked)]
        extern "C" fn wrapper() -> ! {
            naked_asm!(
                save_scratch_reg!(),
                "mov rsi, [rsp + 8*9]
                 mov rdi, rsp
                 add rdi, 8*10
                 sub rsp, 8
                 call {func}
                 add rsp, 8",
                restore_scratch_reg!(),
                "add rsp, 8
                 iretq",
                func = sym $name
            );
        }
        wrapper
    }};
}

#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: VirtAddr,
    code_segment: u64,
    cpu_flags: DebugHex<u64>,
    stack_pointer: VirtAddr,
    stack_segment: u64,
}

lazy_static! {
    static ref IDT: InteruptDescriptorTable = {
        let mut idt = InteruptDescriptorTable::new();
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(3, handler!(breakpoint_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(8, handler_with_error_code!(double_fault_handler))
            .set_stack_index(0);
        idt.set_handler(14, handler_with_error_code!(page_fault_handler));
        idt.set_handler(HardwareInterrupt::Timer as u8, handler!(timer_handler));
        idt.set_handler(
            HardwareInterrupt::Keyboard as u8,
            handler!(keyboard_handler),
        );
        idt
    };
}

#[allow(unused)]
pub fn init_idt() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
}
