#[derive(Debug, Clone, Copy)]
pub enum ElfParserError {
    Not64Bit,
    NotExecutable,
    NotLittleEndian,
    NotUnixCompatible,
    Notx64Compatible,
    UnknownProgramType,
    WrongMagicNumber,
}
