use std::{
    convert::TryFrom
};

use bitflags::bitflags;

use crate::{
    error::SegmentError,
    reader::{Reader},
    addr::Addr,
};

// Reserved inclusive range. Operating system specific.
const LOOS: u32 = 0x6000_0000;
const HIOS: u32 = 0x6FFF_FFFF;
// Reserved inclusive range. Processor specific.
const LOPROC: u32 = 0x7000_0000;
const HIPROC: u32 = 0x7FFF_FFFF;

// Reserved inclusive range. Operating system specific.
const LOOS64: u64 = 0x6000_0000;
const HIOS64: u64 = 0x6FFF_FFFF;
// Reserved inclusive range. Processor specific.
const LOPROC64: u64 = 0x7000_0000;
const HIPROC64: u64 = 0x7FFF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentType {
    /// Program header table entry unused.
    PtNull,
    /// Loadable segment
    PtLoad,
    /// Dynamic linking information
    PtDynamic,
    /// Interpreter information.
    PtInterp,
    /// Auxilary Information
    PtNote,
    /// Reserve
    PtShlib,
    /// Segment containing program header table itself.
    PtPhdr,
    /// Thread-Local storage template.
    PtTls,
    /// Value for specific OS
    PtOsSpecific(u32),
    /// Value for specific processor
    PtProcSpecific(u32),
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
            LOOS..=HIOS => Ok(Self::PtOsSpecific(value)),
            LOPROC..=HIPROC => Ok(Self::PtProcSpecific(value)),
            _ => Err(SegmentError::TypeUnknown(value)),
        }
    }
}

bitflags! {
    /// Structure representing the `p_flags` from the Program Header in an Elf file
    pub struct SegmentFlags: u32 {
        const READ = 0x4;
        const WRITE = 0x2;
        const EXEC = 0x1;
    }
}

impl SegmentFlags {
    pub fn parse(reader: &mut Reader) -> Result<Self, SegmentError> {
        let value = reader.read_u32()?;
        Ok( SegmentFlags::from_bits(value).ok_or(SegmentError::SegmentFlagsParseFailed(value))? )
    }
}

#[derive(Debug)]
pub enum SegmentContents {
    /// Contents for a Dynamic table reffered by `PtDynamic` `ProgramHeader` p_type
    Dynamic(DynamicTable),
    Unknown,
}

#[derive(Debug)]
pub struct DynamicTable(Vec<DynamicEntry>);

impl DynamicTable {
    pub fn parse(bytes: &[u8]) -> Result<Self, SegmentError> {
        let mut reader = Reader::from_bytes(bytes);
        let mut table = vec![];
        // Flags if we reached the null entry or not
        let mut still_got_entries = true;
        while still_got_entries {
            let dynamic_entry = DynamicEntry::parse(&mut reader)?;
            table.push(dynamic_entry);
            if dynamic_entry.d_tag == DynamicTag::Null {
                still_got_entries = false;
            }
        }
        Ok(Self(table))
    }

    pub fn entries(&self) -> &Vec<DynamicEntry> {
        &self.0
    }
}

/// Entry referring to a segment containing the .dynamic section
#[derive(Debug, Copy, Clone)]
pub struct DynamicEntry {
    /// Represents the tag/type of the Dynamic Table entry
    pub d_tag: DynamicTag,
    /// Represent the address of the entry. This can be either an integer, representing
    /// an offset or a size, or it can be a virtual address. These addresses are link-time
    /// virtual addresses, and must be relocated to match the object file's actual load address.
    /// This relocation must be done implicitly
    pub d_un: Addr,
}

impl DynamicEntry {
    pub fn parse(reader: &mut Reader) -> Result<Self, SegmentError> {
        let d_tag = DynamicTag::try_from(reader.read_u64()?)?;
        let d_un = Addr::parse(reader)?;
        
        Ok(Self {
            d_tag,
            d_un
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DynamicTag {
    /// Marks the end of the dynamic array
    Null,
    /// The string table offset of the name of a needed library.
    Needed,
    /// Total size, in bytes, of the relocation entries associated with the procedure
    /// linkage table.
    PltRelSz,
    /// Contains an address associated with the linkage table. The specific meaning of this
    /// field is processor-dependent.
    PltGot,
    /// Address of the symbol hash table. Google it
    Hash,
    /// Address of the dynamic string table
    StrTab,
    /// Address of the dynamic symbol table
    SymTab,
    /// Address of a relocation table with Elf64_Rela entries
    RelA,
    /// Total size, in bytes, of the DT_RELA relocation table,
    RelASz,
    /// Size, in bytes, of each RelA relocation entry.
    RelAEnt,
    /// Total size, in bytes, of the string table.
    StrSz,
    /// Size, in bytes, of each symbol table entry.
    SymEnt,
    /// Address of the initialization function.
    Init,
    /// Address of the termination function.
    Fini,
    /// The string table offset of the name of this shared object
    SoName,
    /// The string table offset of a shared library search path string.
    RPath,
    /// The presence of this dynamic table entry modifies the symbol resolution
    /// algorithm for references within the library. Symbols defined within the 
    /// library are used to resolve references before the dynamic linker searches the
    /// ususal search path. d_un is ignored.
    Symbolic,
    /// Address of a relocation table with Rel entries.
    Rel,
    /// Total size, in bytes, of the Rel relocation table.
    RelSz,
    /// Size, in bytes, of each Rel relocation entry
    RelEnt,
    /// Type of relocation entry used for the procedure linkage table.
    /// The d_un member contains either Rel or RelA.
    PltRel,
    /// Reserved for debugger use.
    Debug,
    /// The presence of this dynamic table entry signals that the relocation table
    /// contains relocations for a non-writable segment
    TextRel,
    /// Address of the relocations associated with the procedure linkage table.
    JmpRel,
    /// The presence of this dynamic table entry signals that the dynamic loader
    /// should process all relocations for this object before transferring control
    /// to the program.
    BindNow,
    /// Pointer to an array of pointers to initialization functions.
    InitArray,
    /// Pointer to an array of pointers to termination functions.
    FiniArray,
    /// Size, in bytes, of the array of initialization functions.
    InitArraySz,
    /// Size, in bytes, of the array of termination functions.
    FiniArraySz,
    /// A range between LoOs and HiOs reserved for environment-specific use.
    OsSpecific(u64),
    /// A range between LoProc and HiProc reserved for processor-specific use.
    ProcSpecific(u64),
}

impl TryFrom<u64> for DynamicTag {
    type Error = SegmentError;
    fn try_from(value: u64) -> Result<DynamicTag, Self::Error> {
        let dynamic_tag = match value {
            0 => Self::Null,
            1 => Self::Needed,
            2 => Self::PltRelSz,
            3 => Self::PltGot,
            4 => Self::Hash,
            5 => Self::StrTab,
            6 => Self::SymTab,
            7 => Self::RelA,
            8 => Self::RelASz,
            9 => Self::RelAEnt,
            10 => Self::StrSz,
            11 => Self::SymEnt,
            12 => Self::Init,
            13 => Self::Fini,
            14 => Self::SoName,
            15 => Self::RPath,
            16 => Self::Symbolic,
            17 => Self::Rel,
            18 => Self::RelSz,
            19 => Self::RelEnt,
            20 => Self::PltRel,
            21 => Self::Debug,
            22 => Self::TextRel,
            23 => Self::JmpRel,
            24 => Self::BindNow,
            25 => Self::InitArray,
            26 => Self::FiniArray,
            27 => Self::InitArraySz,
            28 => Self::FiniArraySz,
            LOOS64..=HIOS64 => Self::OsSpecific(value),
            LOPROC64..=HIPROC64 => Self::ProcSpecific(value),
            _ => return Err(SegmentError::DynamicEntryUnknown(value)),
        };

        Ok(dynamic_tag)
    }
}
