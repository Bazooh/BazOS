use crate::interrupts::ExceptionStackFrame;
use crate::memory::PAGE_SIZE;
use crate::memory::memory_mapper::{KernelMapper, MemoryMapper, MemoryTranslator};
use alloc::boxed::Box;
use common::{hlt_loop, serial_println};
use core::arch::asm;
use x86_64::{PhysAddr, VirtAddr};

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
    pub stack_pointer: VirtAddr, // rsp
    kernel_stack_pointer: VirtAddr,
    page_table_addr: PhysAddr,
    id: ThreadId,
    stack_end: VirtAddr,
    should_die: bool,
}

impl Thread {
    pub fn user(
        id: ThreadId,
        stack_end: VirtAddr,
        kernel_stack_end: VirtAddr,
        entry_point: VirtAddr,
        page_table_addr: PhysAddr,
        args: &[&str],
    ) -> Self {
        Thread {
            stack_pointer: unsafe {
                Self::init_stack(page_table_addr, stack_end, entry_point, args, true)
            },
            kernel_stack_pointer: kernel_stack_end,
            page_table_addr,
            id,
            stack_end,
            should_die: false,
        }
    }

    pub fn kernel(
        id: ThreadId,
        stack_end: VirtAddr,
        kernel_stack_end: VirtAddr,
        entry_point: VirtAddr,
        page_table_addr: PhysAddr,
        args: &[&str],
    ) -> Self {
        Thread {
            stack_pointer: unsafe {
                Self::init_stack(page_table_addr, stack_end, entry_point, args, false)
            },
            kernel_stack_pointer: kernel_stack_end,
            page_table_addr,
            id,
            stack_end,
            should_die: false,
        }
    }

    pub fn after_fork(
        id: ThreadId,
        stack_end: VirtAddr,
        kernel_stack_end: VirtAddr,
        stack_pointer: VirtAddr,
        page_table_addr: PhysAddr,
        rax: u64,
    ) -> Self {
        Thread {
            id,
            page_table_addr,
            stack_pointer: Self::init_forked_stack(page_table_addr, stack_pointer, rax),
            kernel_stack_pointer: kernel_stack_end,
            stack_end,
            should_die: false,
        }
    }

    /// Returns the stack_pointer after initializing the stack with a ready to switch context structure
    unsafe fn init_stack(
        page_table_addr: PhysAddr,
        stack_end: VirtAddr,
        entry_point: VirtAddr,
        args: &[&str],
        user: bool,
    ) -> VirtAddr {
        let page_table = unsafe { MemoryMapper::page_table_from_addr(page_table_addr) };
        fn to_kernel_space(addr: VirtAddr, page_table: &impl MemoryTranslator) -> VirtAddr {
            MemoryMapper::to_virt(page_table.to_phys(addr).unwrap())
        }

        let mut rsp = stack_end;

        // store where string bytes live
        let mut str_data: [(*const u8, usize); 32] = [(core::ptr::null(), 0); 32];
        assert!(args.len() <= str_data.len());

        // 1. copy string bytes
        for (i, s) in args.iter().enumerate().rev() {
            let bytes = s.as_bytes();
            rsp -= bytes.len() as u64;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    to_kernel_space(rsp, &page_table).as_mut_ptr(),
                    bytes.len(),
                );
            }
            str_data[i] = (rsp.as_ptr(), bytes.len());
        }

        // 2. align for fat pointers
        let mut rsp = rsp.align_down(16u64);

        // 3. push &str fat pointers
        for i in (0..args.len()).rev() {
            let (ptr, len) = str_data[i];
            rsp -= 8;
            unsafe { *to_kernel_space(rsp, &page_table).as_mut_ptr() = len };
            rsp -= 8;
            unsafe { *to_kernel_space(rsp, &page_table).as_mut_ptr() = ptr };
        }

        let old_page_table_addr = unsafe { MemoryMapper::switch_to(page_table_addr) };

        let mut context: *mut ThreadContext;
        let mut stack_frame: *mut ExceptionStackFrame;
        let mut frame_rsp: u64;
        let mut new_rsp: u64;
        unsafe {
            asm!(
                // Save the old stack and switch
                "mov {old_rsp}, rsp",
                "mov rsp, {rsp}",

                // Trampoline
                "push {trampoline}",
                "mov {frame_rsp}, rsp",

                // ExceptionStackFrame
                "sub rsp, 5*8",
                "mov {stack_frame}, rsp",
                // "sub {rsp}, 0x8", // Set new_rsp to point to the trampoline

                // All scratch and unscratched registers
                "sub rsp, 15*8",

                "mov {new_rsp}, rsp",
                "mov {context}, rsp",

                // Restore the old stack
                "mov rsp, {old_rsp}",

                trampoline   = in(reg) hlt_loop as *const () as u64,
                frame_rsp   = out(reg) frame_rsp,
                old_rsp      = in(reg) 0u64,
                rsp          = in(reg) rsp.as_u64(),
                new_rsp     = out(reg) new_rsp,
                context     = out(reg) context,
                stack_frame = out(reg) stack_frame,
            )
        };

        let context = unsafe { &mut *context };
        *context = ThreadContext::default();
        context.rdi = rsp.as_u64();
        context.rsi = args.len() as u64;

        let stack_frame = unsafe { &mut *stack_frame };
        *stack_frame = ExceptionStackFrame {
            instruction_pointer: entry_point,
            code_segment: if user { 0x23 } else { 0x8 }, // Entry 0x20 of gdt user code + 0x3 (because DPL = 3)
            cpu_flags: 0x202,
            stack_pointer: VirtAddr::new(frame_rsp),
            stack_segment: if user { 0x1b } else { 0x10 }, // Entry 0x18 of gdt user data + 0x3 (because DPL = 3)
        };

        unsafe { MemoryMapper::switch_to(old_page_table_addr) };

        VirtAddr::new(new_rsp)
    }

    /// Returns the stack_pointer after initializing the stack with a ready to switch context structure
    fn init_forked_stack(page_table_addr: PhysAddr, stack_pointer: VirtAddr, rax: u64) -> VirtAddr {
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
        VirtAddr::new(rsp)
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

    pub fn kernel_stack_pointer(&self) -> VirtAddr {
        self.kernel_stack_pointer
    }

    pub fn should_die(&self) -> bool {
        self.should_die
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        todo!()
    }
}

#[repr(C)]
#[derive(Default)]
struct ThreadContext {
    // Unscratched registers
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    // Scratched registers
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
}
