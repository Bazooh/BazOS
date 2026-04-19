use std::serial_println;

use x86_64::{PhysAddr, VirtAddr};

use crate::fs::elf::error::ElfParserError;

pub const ELF_PROGRAM_HEADER_SIZE: usize = 0x38;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfProgramType {
    Load = 1,
    Dynamic = 2,
    InterpretorPath = 3,
    Note = 4,
    ProgramHeaderTable = 6,
    ThreadLocalStorage = 7,

    GnuExceptionHandlingFrame = 0x6474e550,
    GnuStack = 0x6474e551,
    GnuRelocationReadonly = 0x6474e552,
}

impl TryFrom<u32> for ElfProgramType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ElfProgramType::Load),
            2 => Ok(ElfProgramType::Dynamic),
            3 => Ok(ElfProgramType::InterpretorPath),
            4 => Ok(ElfProgramType::Note),
            6 => Ok(ElfProgramType::ProgramHeaderTable),
            7 => Ok(ElfProgramType::ThreadLocalStorage),
            0x6474e550 => Ok(ElfProgramType::GnuExceptionHandlingFrame),
            0x6474e551 => Ok(ElfProgramType::GnuStack),
            0x6474e552 => Ok(ElfProgramType::GnuRelocationReadonly),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct ElfProgramHeader {
    type_: ElfProgramType,
    flags: u32,
    offset: usize,
    vaddr: VirtAddr,
    file_size: usize,
    mem_size: usize,
    align: usize,
}

impl ElfProgramHeader {
    pub fn parse(header: &[u8; ELF_PROGRAM_HEADER_SIZE]) -> Result<Self, ElfParserError> {
        Ok(ElfProgramHeader {
            type_: ElfProgramType::try_from(u32::from_le_bytes(header[..4].try_into().unwrap()))
                .map_err(|_| ElfParserError::UnknownProgramType)?,
            flags: u32::from_le_bytes(header[4..8].try_into().unwrap()),
            offset: usize::from_le_bytes(header[8..16].try_into().unwrap()),
            vaddr: VirtAddr::new(u64::from_le_bytes(header[16..24].try_into().unwrap())),
            file_size: usize::from_le_bytes(header[32..40].try_into().unwrap()),
            mem_size: usize::from_le_bytes(header[40..48].try_into().unwrap()),
            align: usize::from_le_bytes(header[48..56].try_into().unwrap()),
        })
    }

    pub fn should_load(&self) -> bool {
        self.type_ == ElfProgramType::Load
    }

    pub fn virt_addr(&self) -> VirtAddr {
        self.vaddr
    }

    pub fn mem_size(&self) -> usize {
        self.mem_size
    }

    pub fn align(&self) -> usize {
        self.align
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn file_size(&self) -> usize {
        self.file_size
    }
}
