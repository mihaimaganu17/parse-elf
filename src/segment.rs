use std::{
    fmt,
    convert::TryFrom
};

use crate::{
    error::SegmentError,
    reader::Reader,
};

#[derive(Debug)]
pub enum SegmentType {
    /// Program header table entry unused.
    PtNull = 0x00,
    /// Loadable segment
    PtLoad = 0x01,
    /// Dynamic linking information
    PtDynamic = 0x02,
    /// Interpreter information.
    PtInterp = 0x03,
    /// Auxilary Information
    PtNote = 0x04,
    /// Reserve
    PtShlib = 0x05,
    /// Segment containing program header table itself.
    PtPhdr = 0x06,
    /// Thread-Local storage template.
    PtTls = 0x07,
}

impl SegmentType {
    pub fn parse(reader: &mut Reader) -> Result<Self, SegmentError> {
        let value: u32 = reader.read_u32()?;
        let segment_type: Self = SegmentType::try_from(value)?;
        Ok(segment_type)
    }
}

impl TryFrom<u32> for SegmentType {
    type Error = SegmentError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Self::PtNull),
            0x1 => Ok(Self::PtLoad),
            0x2 => Ok(Self::PtDynamic),
            0x3 => Ok(Self::PtInterp),
            0x4 => Ok(Self::PtNote),
            0x5 => Ok(Self::PtShlib),
            0x6 => Ok(Self::PtPhdr),
            0x7 => Ok(Self::PtTls),
            _ => Err(SegmentError::TypeUnknown(value)),
        }
    }
}

/// Executable Segment flag
const PF_X: u32 = 0x1;
/// Writable Segment flag
const PF_W: u32 = 0x2;
/// Readable Segment flag
const PF_R: u32 = 0x4;

/// Structure representing the `p_flags` from the Program Header in an Elf file
pub struct SegmentFlags {
    pub value: u32,
    pub read: bool,
    pub write: bool,
    pub exec: bool,
}

impl SegmentFlags {
    pub fn parse(reader: &mut Reader) -> Result<Self, SegmentError> {
        let value: u32 = reader.read_u32()?;
        let (mut read, mut write, mut exec) = (false, false, false);

        if value & PF_X == PF_X {
            exec = true;
        }
        if value & PF_W  == PF_W {
            write = true;
        }
        if value & PF_R  == PF_R {
            read = true;
        }

        Ok(Self {
            value,
            read,
            write,
            exec,
        })
    }
}

impl fmt::Debug for SegmentFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
               &[
                    (self.read, "R"),
                    (self.write, "W"),
                    (self.exec, "X"),
               ].iter()
               .map(|&(flag, letter)| {
                   if flag == true {
                       letter
                   } else {
                       "-"
                   }
               }).collect::<Vec<_>>().join(""),)
    }
}
