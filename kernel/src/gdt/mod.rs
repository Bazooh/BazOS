use core::arch::asm;

use bit_field::BitField;
use lazy_static::lazy_static;
use x86_64::{VirtAddr, structures::DescriptorTablePointer};

use crate::gdt::tss::{DOUBLE_FAULT_STACK, TaskStateSegment};

mod tss;

enum Privilege {
    Kernel = 0,
    User = 3,
}

#[derive(Debug, Clone, Copy)]
struct Access(u8);

impl Access {
    /// Returns the access byte for a data segment
    /// `direction` is true if the segment grows down, false if it grows up
    fn data(privilege: Privilege, direction: bool, writable: bool) -> Access {
        let mut access = 0b1001_0000u8;
        access.set_bits(5..=6, privilege as u8);
        access.set_bit(2, direction);
        access.set_bit(1, writable);
        Access(access)
    }

    /// Returns the access byte for a code segment
    /// `conforming` is true if the segment can be executed by any privilege with equal or lower privilege
    /// It is false if the segment can only be executed by the privilege specified by `privilege`
    fn code(privilege: Privilege, conforming: bool, readable: bool) -> Access {
        let mut access = 0b1001_1000u8;
        access.set_bits(5..=6, privilege as u8);
        access.set_bit(2, conforming);
        access.set_bit(1, readable);
        Access(access)
    }

    fn system(privilege: Privilege, type_: Type) -> Access {
        let mut access = 0b1000_0000u8;
        access.set_bits(5..=6, privilege as u8);
        access.set_bits(0..=3, type_ as u8);
        Access(access)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Flags {
    Null = 0b0000,
    LongMode = 0b0010,
}

#[allow(unused)]
#[repr(u8)]
enum Type {
    LDT = 0x2,
    AvailableTSS = 0x9,
    BusyTSS = 0xB,
}

#[derive(Debug, Clone, Copy)]
struct SegmentDescriptor {
    access: Access,
    flags: Flags,
    limit: u32,
}

impl SegmentDescriptor {
    fn new(access: Access, flags: Flags) -> SegmentDescriptor {
        SegmentDescriptor {
            access,
            flags,
            limit: 0xFFFFF,
        }
    }

    fn null() -> SegmentDescriptor {
        SegmentDescriptor {
            access: Access(0),
            flags: Flags::Null,
            limit: 0,
        }
    }

    fn as_u64(self) -> u64 {
        let mut value = 0;
        value.set_bits(52..=55, self.flags as u64);
        value.set_bits(48..=51, (self.limit >> 16) as u8 as u64);
        value.set_bits(40..=47, self.access.0 as u64);
        value.set_bits(0..=15, self.limit as u16 as u64);
        value
    }
}

struct SystemSegmentDescriptor {
    base: u64,
    limit: u64,
    access: Access,
    flags: Flags,
}

impl SystemSegmentDescriptor {
    fn new(base: u64, limit: u64, access: Access, flags: Flags) -> SystemSegmentDescriptor {
        SystemSegmentDescriptor {
            base,
            limit,
            access,
            flags,
        }
    }

    fn as_u128(self) -> u128 {
        let mut value = 0;
        value.set_bits(64..=95, (self.base >> 32) as u32 as u128);
        value.set_bits(56..=63, (self.base >> 24) as u8 as u128);
        value.set_bits(52..=55, self.flags as u128);
        value.set_bits(48..=51, (self.limit >> 16) as u8 as u128);
        value.set_bits(40..=47, self.access.0 as u128);
        value.set_bits(32..=39, (self.base >> 16) as u8 as u128);
        value.set_bits(16..=31, self.base as u16 as u128);
        value.set_bits(0..=15, self.limit as u16 as u128);
        value
    }
}

#[repr(C)]
struct GlobalDescriptorTable {
    table: [u64; 16],
    size: u8,
    tss_position: Option<u8>,
}

impl GlobalDescriptorTable {
    fn new() -> GlobalDescriptorTable {
        let mut gdt = GlobalDescriptorTable {
            table: [0; 16],
            size: 3,
            tss_position: None,
        };
        gdt.table[0] = SegmentDescriptor::null().as_u64();
        gdt.table[1] = SegmentDescriptor::new(
            Access::code(Privilege::Kernel, false, true),
            Flags::LongMode,
        )
        .as_u64();
        gdt.table[2] = SegmentDescriptor::new(
            Access::data(Privilege::Kernel, false, true),
            Flags::LongMode,
        )
        .as_u64();
        gdt
    }

    fn add_tss(&mut self, tss: &'static TaskStateSegment) {
        assert!(self.size < 15, "GDT is full");

        let descriptor = tss.as_descriptor().as_u128();
        self.table[self.size as usize] = descriptor as u64;
        self.table[self.size as usize + 1] = (descriptor >> 64) as u64;
        self.tss_position = Some(self.size);
        self.size += 2;
    }

    unsafe fn reload_segments() {
        unsafe {
            asm!(
                "mov ax, 0x10",
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                "mov ss, ax",
                "push 0x08",
                "lea rax, [rip + 2f]",
                "push rax",
                "retfq",
                "2:",
            );
        }
    }

    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: VirtAddr::from_ptr(self.table.as_ptr()),
            limit: (size_of::<u64>() as u16 * self.size as u16) - 1,
        };
        unsafe {
            asm!(
                "lgdt [{}]",
                in(reg) &ptr
            );
        };
        unsafe { GlobalDescriptorTable::reload_segments() };

        if let Some(tss_position) = self.tss_position {
            let position = tss_position as u16 * 0x8;
            unsafe {
                asm!("mov ax, {:x}", "ltr ax", in(reg) position);
            }
        }
    }
}

lazy_static! {
    #[repr(C, align(16))]
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.set_stack(0, &raw const DOUBLE_FAULT_STACK);
        tss
    };

    #[repr(C, align(16))]
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();
        gdt.add_tss(&TSS);
        gdt
    };
}

#[allow(unused)]
pub fn init_gdt() {
    GDT.load();
}
