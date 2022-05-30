//! Module describing the Section header table and its entries.
use thiserror::Error;

use crate::{Addr, Reader, ParseError};

#[derive(Debug)]
pub struct SectionHeader {
    /// An offset to a string in the .shstrtab section that represents the name of this section.
    sh_name: u32,
    /// Identifies the type of this header. TODO define section header types enum
    sh_type: u32,
    /// Identifies the attributes of the section. TODO define section header attributes enum
    sh_flags: u64,
    /// Virtual address of the section in memory, for sections that are loaded.
    sh_addr: Addr,
    /// Offset of the section in the file image.
    sh_offset: u64,
    /// Size in bytes of the section in the file image. May be 0.
    sh_size: u64,
    /// Contains the section index of an associated section.
    /// This field is used for several purposes, depending on the type of section.
    sh_link: u32,
    /// Contains extra information about the section.
    /// This field is used for several purposes, depending on the type of section.
    sh_info: u32,
    /// Contains the required alignment of the section. This field must be a power of two.
    sh_addralign: u64,
    /// Contains the size, in bytes, of each entry, for sections that contain fixed-size entries.
    /// Otherwise, this field contains zero.
    sh_entsize: u64,
}

impl SectionHeader {
    pub fn parse(reader: &mut Reader) -> Result<SectionHeader, SectionError> {
        let sh_name = reader.read_u32()?;
        let sh_type = reader.read_u32()?;
        let sh_flags = reader.read_u64()?;
        let sh_addr = Addr::from(reader.read_u64()?);
        let sh_offset = reader.read_u64()?;
        let sh_size = reader.read_u64()?;
        let sh_link = reader.read_u32()?;
        let sh_info = reader.read_u32()?;
        let sh_addralign = reader.read_u64()?;
        let sh_entsize = reader.read_u64()?;

        Ok(Self {
            sh_name,
            sh_type,
            sh_flags,
            sh_addr,
            sh_offset,
            sh_size,
            sh_link,
            sh_info,
            sh_addralign,
            sh_entsize,
        })
    }

    pub fn sh_addr(&self) -> Addr {
        self.sh_addr
    }
}

#[derive(Debug, Error)]
pub enum SectionError {
    #[error("Error parsing the section table {0}")]
    ParseError(#[from] ParseError),
}