use crate::gdt::{Access, Flags, Privilege, SystemSegmentDescriptor, Type};
use x86_64::VirtAddr;

#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved0: u32,
    privilege_stack: [u64; 3],
    reserved1: u64,
    interrupt_stack_table: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    io_map_base_address: u16,
}

#[repr(C, align(16))]
pub struct Stack([u8; Stack::SIZE]);

impl Stack {
    const SIZE: usize = 4096 * 5;

    const fn new() -> Stack {
        Stack([0; Stack::SIZE])
    }
}

pub static mut DOUBLE_FAULT_STACK: Stack = Stack::new();

impl TaskStateSegment {
    pub fn new() -> TaskStateSegment {
        TaskStateSegment {
            reserved0: 0,
            privilege_stack: [0; 3],
            reserved1: 0,
            interrupt_stack_table: [0; 7],
            reserved2: 0,
            reserved3: 0,
            io_map_base_address: size_of::<Self>() as u16,
        }
    }

    pub fn as_descriptor(&'static self) -> SystemSegmentDescriptor {
        SystemSegmentDescriptor::new(
            VirtAddr::from_ptr(self).as_u64(),
            size_of::<Self>() as u64 - 1,
            Access::system(Privilege::Kernel, Type::AvailableTSS),
            Flags::LongMode,
        )
    }

    pub fn set_stack(&mut self, index: usize, stack: *mut Stack) {
        let stack_start = VirtAddr::from_ptr(stack).as_u64();
        let stack_end = stack_start + Stack::SIZE as u64;
        self.interrupt_stack_table[index] = stack_end;
    }

    pub unsafe fn set_privilege_stack(&self, index: usize, stack_end: VirtAddr) {
        let ptr = &raw const self.privilege_stack[index] as *mut u64;
        unsafe { core::ptr::write_unaligned(ptr, stack_end.as_u64()) };
    }
}
