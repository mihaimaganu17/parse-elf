use core::{mem::size_of, ops::Range};

use crate::error::ParseError;

pub struct Reader<'a> {
    pub bytes: &'a [u8],
    pub index: usize,
}

impl<'a> Reader<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Reader {bytes, index: 0}
    }

    pub fn seek(&mut self, offset: usize) -> Result<(), ParseError> {
        if offset < 0 || offset >= self.bytes.len() {
            return Err(ParseError::OutOfBounds)
        }

        self.index = offset;

        Ok(())
    }
    pub fn read_slice(&mut self, size: usize) -> Result<&[u8], ParseError> {
        let range = Range { start: self.index, end: self.index + size };
        self.index += size;
        self.read_slice_from(range)
    }

    pub fn read_slice_from(
        &self,
        range: Range<usize>
    ) -> Result<&[u8], ParseError> {
        self.bytes.get(range).ok_or(ParseError::OutOfBounds)
    }

    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        let size = size_of::<u8>();
        let range = Range { start: self.index, end: self.index + size };
        self.index += size;
        let subslice = self.read_slice_from(range)?;
        Ok(u8::from_le_bytes(subslice.try_into().unwrap()))
     }

    pub fn read_u16(&mut self) -> Result<u16, ParseError> {
        let size = size_of::<u16>();
        let range = Range { start: self.index, end: self.index + size };
        self.index += size;
        let subslice = self.read_slice_from(range)?;
        Ok(u16::from_le_bytes(subslice.try_into().unwrap()))
     }

    pub fn read_u32(&mut self) -> Result<u32, ParseError> {
        let size = size_of::<u32>();
        let range = Range { start: self.index, end: self.index + size };
        self.index += size;
        let subslice = self.read_slice_from(range)?;
        Ok(u32::from_le_bytes(subslice.try_into().unwrap()))
     }

    pub fn read_u64(&mut self) -> Result<u64, ParseError> {
        let size = size_of::<u64>();
        let range = Range { start: self.index, end: self.index + size };
        self.index += size;
        let subslice = self.read_slice_from(range)?;
        Ok(u64::from_le_bytes(subslice.try_into().unwrap()))
     }
}
