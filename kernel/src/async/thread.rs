use core::{
    alloc::{GlobalAlloc, Layout},
    arch::{asm, naked_asm},
    ops::{Add, DerefMut},
};

use alloc::vec::Vec;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate,
        page,
    },
};

use crate::{
    r#async::{
        process::{self, Process},
        scheduler::Scheduler,
        thread,
    },
    memory::{MEMORY_MAPPER, PAGE_SIZE, PROGRAM_ALLOCATOR},
};

const STACK_SIZE: usize = 4 * PAGE_SIZE;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Context {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,

    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,

    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    pub cs: u64,
    pub ss: u64,
    pub rflags: u64,
    pub rip: u64,
    pub rsp: u64,
}

#[derive(Debug)]
pub struct Thread {
    pid: u64,
    page_table_addr: PhysAddr,
    stack_end: VirtAddr,
    context: Context,
}

impl Thread {
    fn without_context(pid: u64, page_table_addr: PhysAddr) -> Self {
        Thread {
            pid,
            page_table_addr,
            stack_end: Self::create_stack(page_table_addr),
            context: Context {
                rax: 0,
                rbx: 0,
                rcx: 0,
                rdx: 0,

                rdi: 0,
                rsi: 0,
                rbp: 0,

                r8: 0,
                r9: 0,
                r10: 0,
                r11: 0,
                r12: 0,
                r13: 0,
                r14: 0,
                r15: 0,

                cs: 0,
                ss: 0,
                rflags: 0,
                rip: 0,
                rsp: 0,
            },
        }
    }

    pub fn new(pid: u64, page_table_addr: PhysAddr, entry_point: VirtAddr) -> Self {
        let mut thread = Self::without_context(pid, page_table_addr);
        thread.context.rip = entry_point.as_u64();
        thread
    }

    pub fn fork(&self) {
        let mut thread = Self::without_context(self.pid, self.page_table_addr);
        thread.context = self.context.clone();
        thread.context.rax = thread.pid;
        Scheduler::get().add_thread(thread);
    }

    fn create_stack(page_table_addr: PhysAddr) -> VirtAddr {
        let stack = VirtAddr::from_ptr(unsafe {
            PROGRAM_ALLOCATOR.alloc(Layout::from_size_align(STACK_SIZE, PAGE_SIZE).unwrap())
        });

        let phys_addr = MEMORY_MAPPER
            .get()
            .expect("Memory mapper not initialized")
            .translate_addr(stack)
            .expect("Translation failed");

        let phys_offset = MEMORY_MAPPER.get().unwrap().phys_offset();
        let page_table_addr = phys_offset + page_table_addr.as_u64();
        let mut table = unsafe {
            let page_table = &mut *(page_table_addr.as_mut_ptr());
            OffsetPageTable::new(page_table, phys_offset)
        };

        let number_pages = STACK_SIZE.div_ceil(PAGE_SIZE);
        let page: Page<Size4KiB> = Page::from_start_address(stack).expect("Address not aligned");
        let frame = PhysFrame::from_start_address(phys_addr).expect("Address not aligned");
        for i in 0..number_pages as u64 {
            unsafe {
                table
                    .map_to(
                        page + i,
                        frame + i,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::USER_ACCESSIBLE,
                        PROGRAM_ALLOCATOR.frame_allocator().lock().deref_mut(),
                    )
                    .expect("Mapping failed")
                    .flush();
            }
        }
        stack + STACK_SIZE as u64
    }

    #[unsafe(naked)]
    extern "C" fn exit_trampoline() -> ! {
        naked_asm!("cli", "hlt"); // stop interrupts and hlt
    }

    pub fn exec(&self) -> ! {
        unsafe {
            asm!(
                "mov {tmp}, gs:0
                 mov [{tmp}], {self_ptr}",
                "mov cr3, {cr3}",        // switch page table
                "mov rsp, {rsp}",        // set new stack
                "push {trampoline}",     // push exit trampoline
                "jmp {rip}",             // jump to entry point
                tmp        = in(reg) 0,
                self_ptr   = in(reg) self,
                cr3        = in(reg) self.page_table_addr.as_u64(),
                rsp        = in(reg) self.stack_end.as_u64(),
                trampoline = in(reg) Self::exit_trampoline as *const () as usize,
                rip        = in(reg) self.context.rip,
                options(noreturn)
            )
        };
    }
}

#[macro_export]
macro_rules! checkpoint {
    () => {
        "push rdi
         push rax
         mov rax, gs:0
         mov rax, [rax]

         mov [rax + 0x08], rbx
         mov [rax + 0x10], rcx
         mov [rax + 0x18], rdx

         mov [rax + 0x20], rdi
         mov [rax + 0x28], rsi
         mov [rax + 0x30], rbp

         mov [rax + 0x38], r8
         mov [rax + 0x40], r9
         mov [rax + 0x48], r10
         mov [rax + 0x50], r11
         mov [rax + 0x58], r12
         mov [rax + 0x50], r13
         mov [rax + 0x68], r14
         mov [rax + 0x70], r15

         # mov [rax + 0x78], cs
         # mov [gs:0x80], ss

         mov rdi, [rsp + 0x10]    # rflags
         mov [rax + 0x88], rdi    # rflags

         mov rdi, [rsp + 0x08]    # rip
         mov [rax + 0x90], rdi    # rip

         mov rdi, [rsp + 0x18]    # rsp
         mov [rax + 0x98], rdi    # rsp

         pop rdi                  # rax
         mov [rax], rdi           # rax

         mov rax, rdi
         pop rdi"
    };
}

#[macro_export]
macro_rules! restore {
    () => {
        "mov rax, gs:0
         mov rax, [rax]

         mov rbx, [rax + 0x08]
         mov rcx, [rax + 0x10]
         mov rdx, [rax + 0x18]

         mov rdi, [rax + 0x20]
         mov rsi, [rax + 0x28]
         mov rbp, [rax + 0x30]

         mov r8, [rax + 0x38]
         mov r9, [rax + 0x40]
         mov r10, [rax + 0x48]
         mov r11, [rax + 0x50]
         mov r12, [rax + 0x58]
         mov r13, [rax + 0x60]
         mov r14, [rax + 0x68]
         mov r15, [rax + 0x70]

         mov rax, [rax]"
    };
}
