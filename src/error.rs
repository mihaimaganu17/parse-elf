use crate::{
    file_type,
    machine,
};

#[derive(Debug)]
pub enum ElfError {
    ElfHeader(ElfHeaderError),
    ProgramHeader(ProgramHeaderError),
    ParseError(ParseError),
}

impl From<ElfHeaderError> for ElfError {
    fn from(err: ElfHeaderError) -> Self {
        Self::ElfHeader(err)
    }
}

impl From<ProgramHeaderError> for ElfError {
    fn from(err: ProgramHeaderError) -> Self {
        Self::ProgramHeader(err)
    }
}

impl From<ParseError> for ElfError {
    fn from(err: ParseError) -> Self {
        Self::ParseError(err)
    }
}

#[derive(Debug)]
pub enum ElfHeaderError {
    BadMagic(String),
    Not64Bit,
    BadEndianness,
    BadVersion,
    BadOsAbi,
    FileTypeError(file_type::Error),
    MachineError(machine::Error),
    NotOriginalVersion,
    ParseError(ParseError)
}

#[derive(Debug)]
pub enum ParseError {
    OutOfBounds,
}

impl From<ParseError> for ElfHeaderError {
    fn from(err: ParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<file_type::Error> for ElfHeaderError {
    fn from(err: file_type::Error) -> ElfHeaderError {
        ElfHeaderError::FileTypeError(err)
    }
}

impl From<machine::Error> for ElfHeaderError {
    fn from(err: machine::Error) -> ElfHeaderError {
        ElfHeaderError::MachineError(err)
    }
}

#[derive(Debug)]
pub enum ProgramHeaderError {
    SegmentError(SegmentError),
    ParseError(ParseError),
}

impl From<SegmentError> for ProgramHeaderError {
    fn from(err: SegmentError) -> Self {
        Self::SegmentError(err)
    }
}

impl From<ParseError> for ProgramHeaderError {
    fn from(err: ParseError) -> Self {
        Self::ParseError(err)
    }
}

#[derive(Debug)]
pub enum SegmentError {
    TypeUnknown(u32),
    ParseError(ParseError),
}

impl From<ParseError> for SegmentError {
    fn from(err: ParseError) -> Self {
        Self::ParseError(err)
    }
}
