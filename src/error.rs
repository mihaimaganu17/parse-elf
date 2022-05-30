use core::ops::Range;
use thiserror::Error;

use crate::{
    section::SectionError,
    file_type,
    machine,
    DynamicTag,
    addr,
    reloc::Error as RelocError,
};

#[derive(Debug, Error)]
pub enum ElfError {
    #[error("Elf header error {0}")]
    ElfHeader(#[from] ElfHeaderError),
    #[error("Program header error {0}")]
    ProgramHeader(#[from] ProgramHeaderError),
    #[error("Parsing error {0}")]
    ParseError(#[from] ParseError),
    #[error("Section header error {0}")]
    SectionError(#[from] SectionError),
}

#[derive(Debug, Error)]
pub enum ElfHeaderError {
    #[error("Cannot find elf magic, found: {0}")]
    BadMagic(String),
    #[error("Elf is not 64-bit")]
    Not64Bit,
    #[error("Elf is not Littel Endian")]
    BadEndianness,
    #[error("Elf has bad version(not 1)")]
    BadVersion,
    #[error("Unknown OS ABI")]
    BadOsAbi,
    #[error("Unknown object file type {0}")]
    FileTypeError(#[from] file_type::Error),
    #[error("Unknown machine: {0}")]
    MachineError(#[from] machine::Error),
    #[error("Not original version")]
    NotOriginalVersion,
    #[error("Parsing error {0}")]
    ParseError(#[from] ParseError)
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Trying to parse more than the upper bound")]
    OutOfBounds,
    #[error("Trying to access bad range {0:?} from slice")]
    BadRange(Range<usize>),
}

#[derive(Debug, Error)]
pub enum DynamicError {
    #[error("Dynamic Tag not found {0:?}")]
    TagNotFound(DynamicTag),
    #[error("Dynamic Entry unknow {0}")]
    EntryUnknown(u64),
}

#[derive(Debug, Error)]
pub enum ProgramHeaderError {
    #[error("Segment error {0}")]
    SegmentError(#[from] SegmentError),
    #[error("Parse error {0}")]
    ParseError(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum SegmentError {
    #[error("Segment type unknown {0}")]
    TypeUnknown(u32),
    #[error("Segment parsing error {0}")]
    ParseError(#[from] ParseError),
    #[error("Segment flags parsing failed {0}")]
    SegmentFlagsParseFailed(u32),
    #[error("Address not found in any PtLoad segment {0}")]
    BadPtLoadAddr(addr::Addr),
    #[error("Dynamic segment error {0}")]
    DynamicError(#[from] DynamicError),
    #[error("Address Error {0}")]
    AddrError(#[from] addr::Error),
    #[error("Relocation Error {0}")]
    RelocError(#[from] RelocError),
    #[error("String table error: {0}")]
    StrTabError(#[from] StringError),
}

#[derive(Debug, Error)]
pub enum StringError {
    #[error("String Table not found")]
    StrTabNotFound,
    #[error("String Table Segment not found")]
    StrTabSegmentNotFound,
    #[error("String from string Table not found")]
    StringNotFound
}

