use std::serial_println;

use alloc::vec::Vec;
use x86_64::VirtAddr;

use crate::{
    fs::elf::{
        error::ElfParserError,
        program_header::{ELF_PROGRAM_HEADER_SIZE, ElfProgramHeader},
    },
    print_data,
};

pub const ELF_HEADER_SIZE: usize = 0x40;

#[derive(Debug)]
pub struct ElfHeader {
    entry_point: VirtAddr,
    program_headers: Vec<ElfProgramHeader>,
}

impl ElfHeader {
    pub fn parse(content: &[u8]) -> Result<Self, ElfParserError> {
        ElfHeader::check_compatibility(content)?;

        let n_program_headers = u16::from_le_bytes(content[56..58].try_into().unwrap());
        let program_headers = (0..n_program_headers)
            .map(|i| {
                let offset = ELF_HEADER_SIZE + ELF_PROGRAM_HEADER_SIZE * i as usize;
                ElfProgramHeader::parse(
                    &content[offset..offset + ELF_PROGRAM_HEADER_SIZE]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect::<Result<_, _>>()?;

        let entry_point = VirtAddr::new(u64::from_le_bytes(content[24..32].try_into().unwrap()));

        Ok(ElfHeader {
            entry_point,
            program_headers,
        })
    }

    fn check_compatibility(content: &[u8]) -> Result<(), ElfParserError> {
        if &content[0..4] != b"\x7fELF" {
            return Err(ElfParserError::WrongMagicNumber);
        }

        if content[4] != 2 {
            return Err(ElfParserError::Not64Bit);
        }

        if content[5] != 1 {
            return Err(ElfParserError::NotLittleEndian);
        }

        if content[7] != 0 {
            return Err(ElfParserError::NotUnixCompatible);
        }

        if &content[16..=17] != &[0x02, 0x00] {
            return Err(ElfParserError::NotExecutable);
        }

        if &content[18..=19] != &[0x3e, 0x00] {
            return Err(ElfParserError::Notx64Compatible);
        }

        Ok(())
    }

    pub fn program_headers(&self) -> impl Iterator<Item = &ElfProgramHeader> {
        self.program_headers.iter()
    }

    pub fn n_program_headers(&self) -> usize {
        self.program_headers.len()
    }

    pub fn entry_point(&self) -> VirtAddr {
        self.entry_point
    }
}
