use thiserror::Error;

use crate::{
    addr::Addr,
    reader::Reader,
    error::SegmentError,
};

/// Structure of a relocation entry. Rela entries contain an explicit addend.
/// 64-bit x86 use only Rela relocation entries.
#[derive(Debug)]
pub struct Rela {
    /// Gives the location at which to apply the relocation action.
    /// For an executable or shared object, the value indicates the virtual address
    /// of the storage unit affected by the relocation. This information makes the
    /// relocation entries more useful for the runtime linker.
    pub r_offset: Addr,
    /// The type of relocation to apply
    pub r_type: RelType,
    /// Symbol table index, with respect to which the relocation must be made
    pub r_sym: u32,
    /// This member specifies a contant addend used to compute the value to be stored
    /// into th relocatable field.
    pub r_addend: u64,
}

impl Rela {
    pub fn parse(reader: &mut Reader) -> Result<Self, SegmentError> {
        let r_offset = Addr::from(reader.read_u64()?);
        let r_type = RelType::try_from(reader.read_u32()?)?;
        let r_sym = reader.read_u32()?;
        let r_addend = reader.read_u64()?;

        Ok(Self {
            r_offset,
            r_type,
            r_sym,
            r_addend
        })
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelType {
    None,
    W64,
    Pc32,
    Got32,
    Plt32,
    Copy,
    GlobDat,
    JumpSlot,
    Relative,
}

impl TryFrom<u32> for RelType {
    type Error = Error;
    fn try_from(value: u32) -> Result<RelType, Self::Error> {
        let rel_type = match value {
            0 => Self::None,
            1 => Self::W64,
            2 => Self::Pc32,
            3 => Self::Got32,
            4 => Self::Plt32,
            5 => Self::Copy,
            6 => Self::GlobDat,
            7 => Self::JumpSlot,
            8 => Self::Relative,
            _ => return Err(Error::InvalidRelocationType(value)),
        };

        Ok(rel_type)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown relocation type referenced by value {0}")]
    InvalidRelocationType(u32),
}