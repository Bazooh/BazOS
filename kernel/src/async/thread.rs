use crate::memory::{MEMORY_MAPPER, PAGE_SIZE};
use core::arch::{asm, naked_asm};
use core::ops::Deref;
use std::serial_println;
use x86_64::{PhysAddr, VirtAddr, structures::paging::Translate};

pub const STACK_SIZE: u64 = 3 * PAGE_SIZE;

pub const USER_STACK_START: VirtAddr = VirtAddr::new(0xffff900000000000);

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
#[repr(C)]
pub struct ThreadId {
    pub pid: u64,
    pub thread_id: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct Thread {
    stack_pointer: VirtAddr, // rsp
    page_table_addr: PhysAddr,
    id: ThreadId,
}

impl Thread {
    pub fn new(
        id: ThreadId,
        stack_end: VirtAddr,
        entry_point: VirtAddr,
        page_table_addr: PhysAddr,
    ) -> Self {
        Thread {
            id,
            page_table_addr,
            stack_pointer: Self::init_stack(page_table_addr, stack_end, entry_point),
        }
    }

    pub fn after_fork(
        id: ThreadId,
        stack_pointer: VirtAddr,
        page_table_addr: PhysAddr,
        rax: u64,
    ) -> Self {
        Thread {
            id,
            page_table_addr,
            stack_pointer: Self::init_forked_stack(page_table_addr, stack_pointer, rax),
        }
    }

    pub fn kernel() -> Self {
        Thread {
            id: ThreadId {
                pid: 0,
                thread_id: 0,
            },
            stack_pointer: VirtAddr::zero(),
            page_table_addr: MEMORY_MAPPER
                .translate_addr(VirtAddr::from_ptr(MEMORY_MAPPER.deref()))
                .unwrap(),
        }
    }

    /// Returns the stack_pointer after initializing the stack with a ready to switch context structure
    fn init_stack(
        page_table_addr: PhysAddr,
        stack_end: VirtAddr,
        entry_point: VirtAddr,
    ) -> VirtAddr {
        serial_println!("switching to new page table: {:?}", page_table_addr);

        let mut rsp: u64 = stack_end.as_u64();
        unsafe {
            asm!(
                // Save the old page table and switch
                "mov {old_cr3}, cr3",
                "mov cr3, {cr3}",
                // Save the old stack and switch
                "mov {old_rsp}, rsp",
                "mov rsp, {rsp}",

                // Trampoline
                "push {trampoline}",

                // ExceptionStackFrame
                "push {ss}",
                "sub {rsp}, 0x8", // Set new_rsp to point to the trampoline
                "push {rsp}",
                "push {rflags}",
                "push {cs}",
                "push {rip}",

                // All scratch and unscratched registers
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",
                "push {zero}",

                "mov {rsp}, rsp",

                // Restore the old stack
                "mov rsp, {old_rsp}",
                // Restore the old page table
                "mov cr3, {old_cr3}",

                trampoline = in(reg) Self::exit_trampoline as *const () as u64,
                rip        = in(reg) entry_point.as_u64(),
                cs         = in(reg) 0x8u64,
                rflags     = in(reg) 0x202u64,
                ss         = in(reg) 0x10u64,
                zero       = in(reg) 0u64,
                old_cr3    = in(reg) 0u64,
                cr3        = in(reg) page_table_addr.as_u64(),
                old_rsp    = in(reg) 0u64,
                rsp     = inout(reg) rsp,
            )
        };

        serial_println!("entry point: {:?}", entry_point);
        VirtAddr::new(rsp)
    }

    /// Returns the stack_pointer after initializing the stack with a ready to switch context structure
    fn init_forked_stack(page_table_addr: PhysAddr, stack_pointer: VirtAddr, rax: u64) -> VirtAddr {
        serial_println!("switching to new page table: {:?}", page_table_addr);

        let mut rsp: u64 = stack_pointer.as_u64();
        unsafe {
            asm!(
            // Save the old page table and switch
            "mov {old_cr3}, cr3",
            "mov cr3, {cr3}",
            // Save the old stack and switch
            "mov {old_rsp}, rsp",
            "mov rsp, {rsp}",

            // All scratch and unscratched registers
            "push {rax}",
            "sub rsp, 8*8",
            crate::save_unscratched_reg!(),

            "mov {rsp}, rsp",

            // Restore the old stack
            "mov rsp, {old_rsp}",
            // Restore the old page table
            "mov cr3, {old_cr3}",

            rax        = in(reg) rax,
            old_cr3    = in(reg) 0u64,
            cr3        = in(reg) page_table_addr.as_u64(),
            old_rsp    = in(reg) 0u64,
            rsp     = inout(reg) rsp,
            )
        };

        serial_println!("entry point");

        VirtAddr::new(rsp)
    }

    #[unsafe(naked)]
    extern "C" fn exit_trampoline() -> ! {
        // TODO Clean the thread
        naked_asm!("idle:", "hlt", "jmp idle")
    }

    pub fn page_table_addr(&self) -> PhysAddr {
        self.page_table_addr
    }

    pub fn id(&self) -> ThreadId {
        self.id
    }

    pub fn pid(&self) -> u64 {
        self.id.pid
    }

    pub fn stack_pointer(&self) -> VirtAddr {
        self.stack_pointer
    }
}
