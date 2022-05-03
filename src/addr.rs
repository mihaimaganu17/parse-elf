//! Module that defines a 64-bit address
use core::fmt;

use crate::{error::ParseError, reader};

#[derive(Copy, Clone, PartialEq)]
pub struct Addr(pub u64);

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Use for serializing
impl Into<u64> for Addr {
    fn into(self) -> u64 {
        self.0
    }
}

/// Used for indexing and subslicing
impl Into<usize> for Addr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

/// Used for parsing
impl From<u64> for Addr {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Addr {
    pub fn parse(reader: &mut reader::Reader) -> Result<Self, ParseError> {
        let value = reader.read_u64()?;
        Ok(Self(value))
    }
}
