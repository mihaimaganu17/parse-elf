//! Module describing and parsing the symbol table of Elf files
use thiserror::Error;

use crate::{
    Addr,
};

/// Lower bound for OS specific use
const LOOS: u8 = 10;
/// Higher bound for OS specific use
const HIOS: u8 = 12;
/// Lower bound for processor specific use
const LOPROC: u8 = 13;
/// Higher bound for processor specific use
const HIPROC: u8 = 15;

/// Section index used to mark an undefined or meaningless section reference
const SHN_UNDEF: u16 = 0;
/// Section index used to indicate that the corresponding reference is an absolute value
const SHN_ABS: u16 = 0xFFF1;
/// Section index used to indicate a symbol that has been declared a common block
/// (Fortran COMMON or C tentatic declaration)
const SHN_COMMON: u16 = 0xFFF2;

/// The first sybol table entry is reserved and must be all zeroes.
/// The symbolic constant STN_UNDEF is used to refer to this entry.
pub struct SymbolEntry {
    /// Contains the offset, in bytes, to the symbol name, relatice to the start of the symbol
    /// string table. If this field contains zero, the symbol has no name.
    st_name: u32,
    /// Contains the symbol type and its binding attributes
    st_info: SymbolInfo,
    /// Reserved for future use; must be zero
    st_other: u8,
    /// Section table index of the section in which the symbol is defined. For undefined symbols,
    /// this field contains `SHN_UNDEF`; For absolute symbols, it contains `SHN_ABS`; and for
    /// common symbols, it contains `SHN_COMMON`.
    st_shndx: u16,
    /// Contains the value of the symbol. This may be an absolute value or a relocatable address.
    st_value: Addr,
    /// Contains the size associated with the symbol. If a symbol does not have an associated size,
    /// or the size is unknown, this field contains zero.
    st_size: u64,
}

impl SymbolEntry {
    pub fn parse(reader: &mut Reader) -> Result<Self, SymbolError> {
        let st_name = reader.read_u32()?;
        let st_info = SymbolInfo::try_from(reader.read_u8())?;
        let st_other = reader.read_u8()?;
        let st_shndx = reader.read_u16()?;
        let st_value = Addr::from(reader.read_u64()?);
        let st_size = reader.read_u64()?;
        Ok(Self {
            st_name,
            st_info,
            st_other,
            st_shndx,
            st_value,
            st_size,
        })
    }
}

/// Information regarding a symbol table entry.
pub struct SymbolInfo {
    /// Type attributes contained in the low-order four bits.
    st_type: SymbolType,
    /// Binding attributes contained in the high-order four bits of the eight-bit byte
    st_binding: SymbolBinding,
}

impl TryFrom<u8> for SymbolInfo {
    type Error = SymbolError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let low_bits = value & 0xF;
        let high_bits = value >> 4;
        let st_type = SymbolType::try_from(low_bits)?;
        let st_binding = SymbolBinding::try_from(high_bits)?;
        Ok(Self { st_type, st_binding })
    }
}

pub enum SymbolType {
    NoType,
    Object,
    Func,
    Section,
    File,
    OsSpecific(u8),
    ProcSpecific(u8),
}

pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    OsSpecific(u8),
    ProcSpecific(u8),
}

impl TryFrom<u8> for SymbolType {
    type Error = SymbolError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Self::NoType,
            1 => Self::Object,
            2 => Self::Func,
            3 => Self::Section,
            4 => Self::File,
            LOOS..=HIOS => OsSpecific(value),
            LOPROC..=HIPROC => ProcSpecific(value),
            _ => return Err(SymbolError::UnknownSymbolType(value))
        }
    }
}

impl TryFrom<u8> for SymbolBinding {
    type Error = SymbolError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Self::Local,
            1 => Self::Global,
            2 => Self::Weak,
            LOOS..=HIOS => OsSpecific(value),
            LOPROC..=HIPROC => ProcSpecific(value),
            _ => return Err(SymbolError::UnknownSymbolBinding(value))
        }
    }
}

#[derive(Debug, Error)]
pub enum SymbolError {
    #[error("Symbol type referenced by value {0} is unknown")]
    UnknownSymbolType(u8),
    #[error("Symbol binding referenced by value {0} is unknown")]
    UnknownSymbolBinding(u8),
}
