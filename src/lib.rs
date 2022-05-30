use std::{fmt, ops::Range};

pub mod addr;
pub mod error;
pub mod file_type;
pub mod machine;
pub mod segment;
pub mod section;
pub mod reader;
pub mod reloc;

use segment::DynamicEntry;
pub use segment::{SegmentContents, DynamicTable};

pub use crate::{
    addr::Addr,
    error::{
        ElfError,
        ElfHeaderError,
        ProgramHeaderError,
        SegmentError,
        ParseError,
        DynamicError,
        StringError,
    },
    file_type::FileType,
    machine::Machine,
    segment::{SegmentType, SegmentFlags, DynamicTag},
    reloc::{Rela, RelType},
    reader::Reader,
    section::{SectionHeader},
};

/// Structure that represents an Elf 64-bit file
/// We are only parsing x86 ISA little endian Elfs
pub struct Elf64 {
    pub elf_header: ElfHeader,
    /// `ProgramHeader` table
    pub ph_table: Vec<ProgramHeader>,
    /// `SectionHeader` table
    pub sh_table: Vec<SectionHeader>,
}

impl Elf64 {
    pub fn parse(bytes: &[u8]) -> Result<Self, ElfError> {
        let mut reader = Reader::from_bytes(bytes);
        let elf_header = ElfHeader::parse(&mut reader)?;

        // Allocate a new vector to hold the Program header table
        let mut ph_table = Vec::with_capacity(elf_header.e_phnum().into());

        // Move the read cursor to the program header table beginning
        reader.seek(elf_header.e_phoff().into())?;

        for _ in 0..elf_header.e_phnum() {
            ph_table.push(ProgramHeader::parse(&mut reader)?);
        }

        // Allocate a new vector to hold the SectionHeader table
        let mut sh_table = Vec::with_capacity(elf_header.e_shnum().into());
        // Move the read cursor to the section header table beginning
        reader.seek(elf_header.e_shoff().into())?;

        for _ in 0..elf_header.e_shnum() {
            sh_table.push(SectionHeader::parse(&mut reader)?);
        }

        Ok(Self {
            elf_header,
            ph_table,
            sh_table,
        })
    }

    /// Returns the `ProgramHeader` of the segment that contains the `addr`
    pub fn segment_at(&self, addr: Addr) -> Option<&ProgramHeader> {
        self.ph_table
            .iter()
            .filter(|ph| ph.p_type == SegmentType::PtLoad)
            .find(|ph| ph.mem_range().contains(&addr))
    }

    /// Returns a slice from the the Load segment containing `mem_addr` address.
    /// The slice spans from `mem_addr` until the end of the segment.
    pub fn slice_at(&self, mem_addr: Addr) -> Option<&[u8]> {
        self.segment_at(mem_addr)
            .map(|seg| &seg.data[(mem_addr - seg.mem_range().start).into()..])
    }

    /// Returns a string from the string table located at `offset`.
    pub fn get_string(&self, offset: Addr) -> Result<String, StringError> {
        let addr = self.dynamic_entry(DynamicTag::StrTab).ok_or(StringError::StringNotFound)?;
        let slice = self
            .slice_at(addr + offset)
            .ok_or(StringError::StrTabSegmentNotFound)?;
        // String are null terminated. So we split the slice into slices separated by '\0'
        let string_slice = slice.split(|&c| c == 0).next().ok_or(StringError::StringNotFound)?;
        Ok(String::from_utf8_lossy(string_slice).into())
    }

    /// Returns the first segment of type `p_type`.
    pub fn segment_of_type(&self, p_type: SegmentType) -> Option<&ProgramHeader> {
        self.ph_table
            .iter()
            .find(|ph| ph.p_type() == p_type)
    }

    /// Return an entry from the Dynamic table with the given `tag` or None if `tag` does not exist
    /// in the table
    pub fn dynamic_entry(&self, tag: DynamicTag) -> Option<Addr> {
        self.dynamic_entries(tag).next()
    }

    pub fn dynamic_table(&self) -> Option<&[DynamicEntry]> {
        match self.segment_of_type(SegmentType::PtDynamic) {
            Some(ProgramHeader {
                contents: SegmentContents::Dynamic(table),
                ..
            }) => Some(table.entries()),
            _ => None,
        }
    }

    /// Returns an `Interator` over addresses contained in the entries of the dynamic table which
    /// have `tag`
    pub fn dynamic_entries(&self, tag: DynamicTag) -> impl Iterator<Item = Addr> + '_{
        self.dynamic_table()
            .unwrap_or_default()
            .iter()
            .filter(move |e| e.d_tag == tag)
            .map(|e| e.d_un)
    }

    pub fn dynamic_entry_strings(&self, tag: DynamicTag) -> impl Iterator<Item = String> + '_ {
        self.dynamic_entries(tag)
            .filter_map(move |addr| self.get_string(addr).ok())
    }

    /// Reads and returns the vector of `Rela` entries from the file
    pub fn read_rela_entries(&self) -> Result<Vec<Rela>, SegmentError> {
        use DynamicTag;
        use DynamicError;

        // Get address for the Rela entries
        let rela_addr = self
            .dynamic_entry(DynamicTag::RelA)
            .ok_or(DynamicError::TagNotFound(DynamicTag::RelA))?;

        // Get total length, in bytes, for the Rela entries
        let rela_len = self
            .dynamic_entry(DynamicTag::RelASz)
            .ok_or(DynamicError::TagNotFound(DynamicTag::RelASz))?;

        // Get the segment where the Rela entries are store
        let seg = self.segment_at(rela_addr).ok_or(SegmentError::BadPtLoadAddr(rela_addr))?;

        // Prepare a range to fetch bytes
        let rela_range: Range<usize> = Range { 
            start: (rela_addr - seg.mem_range().start).into(),
            end: ((rela_addr + rela_len) - seg.mem_range().start).into(),
        };

        // Fetch the slice to parse the rela from
        let rela_slice = seg.data.get(rela_range.clone()).ok_or(ParseError::BadRange(rela_range))?;

        // Construct a reader
        let mut reader = Reader::from_bytes(rela_slice);

        // Initialise a `Vec` to hold Rela entries
        let mut rela_entries: Vec<Rela> = vec![];
        // Parse the Rela entries
        while reader.index < rela_len.into() {
            let rela = Rela::parse(&mut reader)?;
            rela_entries.push(rela);
        }

        Ok(rela_entries)
        
    }

    /// Returns the section header that start at EXACTLY this virtual address `addr`,
    /// or `None` if we can't find one.
    pub fn section_starting_at(&self, addr: Addr) -> Option<&SectionHeader> {
        self.sh_table.iter().find(|&sh| sh.sh_addr() == addr)
    }
}

impl fmt::Debug for Elf64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}\n{}\n",
            self.elf_header,
            self.ph_table.iter().map(|ph_entry| format!("{:?}", ph_entry))
            .collect::<Vec<_>>().join("\n"))
    }
}

/// Tell the system how to create a process image. It is found at file offset
/// `e_phoff` and consists of `e_phnum` entries, each with size `e_phentsize`.
pub struct ProgramHeader {
    /// Identifies the type of the segment
    p_type: SegmentType,
    /// BitMask for segment-dependent flags
    p_flags: SegmentFlags,
    /// Offset of the segment in the file image,
    p_offset: Addr,
    /// Virtual Address of the segment in memory,
    p_vaddr: Addr,
    /// On systems where physical address is relevant, reserved for segment's
    /// physical address
    p_paddr: Addr,
    /// Size in bytes of the segment in the file image. May be 0.
    p_filesz: Addr,
    /// Size in bytes of the segment in memory
    p_memsz: Addr,
    /// 0 and 1 specify no alignment. Otherwise should be a positive, integral
    /// power of 2 with p_vaddr = p_offset % p_align
    p_align: Addr,
    /// A vector storing the contents of the segment
    pub data: Vec<u8>,
    /// Contents of the current segment based on `SegmentType`
    pub contents: SegmentContents,
}

impl ProgramHeader {
    pub fn parse(reader: &mut Reader) -> Result<Self, ProgramHeaderError> {
        let p_type = SegmentType::parse(reader)?;
        let p_flags = SegmentFlags::parse(reader)?;
        let p_offset = Addr::parse(reader)?;
        let p_vaddr = Addr::parse(reader)?;
        let p_paddr = Addr::parse(reader)?;
        let p_filesz = Addr::parse(reader)?;
        let p_memsz = Addr::parse(reader)?;
        let p_align = Addr::parse(reader)?;

        let segment_start: usize = p_offset.into();
        let segment_end: usize = Into::<usize>::into(p_offset) +
            Into::<usize>::into(p_filesz);

        let segment_data_range = Range {
            start: segment_start,
            end: segment_end
        };

        let data = reader.read_slice_from(segment_data_range)?.to_vec();

        let contents = match p_type {
            SegmentType::PtDynamic => {
                // Parse the dynamic table
                SegmentContents::Dynamic(DynamicTable::parse(&data)?)
            },
            _ => SegmentContents::Unknown,
        };

        Ok(Self {
            p_type,
            p_flags,
            p_offset,
            p_vaddr,
            p_paddr,
            p_filesz,
            p_memsz,
            p_align,
            data,
            contents,
        })
    }

    /// Returns a range where the segment is stored in the file
    pub fn file_range(&self) -> Range<Addr> {
        self.p_offset..self.p_offset + self.p_filesz
    }

    /// Returns a range where the segment should be stored in memory
    pub fn mem_range(&self) -> Range<Addr> {
        self.p_vaddr..self.p_vaddr + self.p_memsz
    }

    pub fn p_vaddr(&self) -> Addr {
        self.p_vaddr
    }

    pub fn p_memsz(&self) -> Addr {
        self.p_memsz
    }

    pub fn p_flags(&self) -> SegmentFlags {
        self.p_flags
    }

    pub fn p_type(&self) -> SegmentType {
        self.p_type
    }

    pub fn p_align(&self) -> Addr {
        self.p_align
    }

    pub fn p_addr(&self) -> Addr {
        self.p_paddr
    }
}

impl fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            " Type: {:?}\n Flags: {:?}\n Contents: {:?}\n",
            self.p_type,
            self.p_flags,
            self.contents,
        )
    }
}

const ELF_MAGIC_SIZE: usize = 4;
const ELF_MAGIC: &[u8] = &[0x7F, 0x45, 0x4C, 0x46];

#[derive(Debug)]
pub struct ElfHeader {
    pub e_type: FileType,
    pub e_machine: Machine,
    /// Memory address of the entry point from where the process starts
    /// executing
    pub e_entry: Addr,
    /// Points to the start of the program header table.
    pub e_phoff: Addr,
    /// Points to the start of the section header table.
    pub e_shoff: Addr,
    /// Contains the size of a program header table entry.
    pub e_phentsize: u16,
    /// Contains the number of entries in the program header table.
    pub e_phnum: u16,
    /// Contains the size of a section header table entry.
    pub e_shentsize: u16,
    /// Contains the number of entries in the section header table.
    pub e_shnum: u16,
    /// Contains index of the section header table entry that contains the section names.
    pub e_shstrndx: u16,
}

impl ElfHeader {
    pub fn parse(reader: &mut Reader) -> Result<Self, ElfHeaderError> {
        // Read the magic
        let e_magic = reader.read_slice(ELF_MAGIC_SIZE)?;
        // Check if we have an Elf files
        if e_magic != ELF_MAGIC {
            return Err(ElfHeaderError::BadMagic(format!("{:?}", e_magic)))
        }

        // Read the class
        let e_class = reader.read_u8()?;
        // Check the class is 64-bit
        if e_class != 2 {
            return Err(ElfHeaderError::Not64Bit)
        }

        // Read the endianness
        let e_data = reader.read_u8()?;
        // Check that is little endian
        if e_data != 1 {
            return Err(ElfHeaderError::BadEndianness)
        }

        // Read the version
        let e_version = reader.read_u8()?;
        // Should be 1 for the original and current version of Elf
        if e_version != 1 {
            return Err(ElfHeaderError::BadVersion)
        }

        // Read the target operating system ABI
        let e_osabi = reader.read_u8()?;
        // Check the OS Abi is System V or Linux
        if e_osabi != 0 && e_osabi != 3 {
            return Err(ElfHeaderError::BadOsAbi)
        }

        // Skip the remaining padding
        let _ = reader.read_slice(8)?;

        // Read the object file_type
        let e_type: FileType = reader.read_u16()?.try_into()?;

        // Read the object machine
        let e_machine: Machine = reader.read_u16()?.try_into()?;

        // Read yet another version
        let e_version = reader.read_u32()?;

        // Check if version has the only value possible
        if e_version != 1 {
            return Err(ElfHeaderError::NotOriginalVersion);
        }

        // Read entry point
        let e_entry = Addr::parse(reader)?;


        // Read the offset of the Program Header table
        let e_phoff = Addr::parse(reader)?;

        // Read start of the section header table
        let e_shoff = Addr::parse(reader)?;

        // Skip `e_flags` 4-bytes and `e_ehsize` 2-bytes
        let _ = reader.read_slice(6)?;

        // Read the size of a Program Header table entry.
        let e_phentsize = reader.read_u16()?;
        // Read Program Header table entries
        let e_phnum = reader.read_u16()?;

        // Read information about the section header table
        let e_shentsize = reader.read_u16()?;
        let e_shnum = reader.read_u16()?;
        let e_shstrndx = reader.read_u16()?;


        Ok(ElfHeader{
            e_type,
            e_machine,
            e_entry,
            e_phoff,
            e_shoff,
            e_phentsize,
            e_phnum,
            e_shentsize,
            e_shnum,
            e_shstrndx,
        })
    }

    pub fn e_phoff(&self) -> Addr {
        self.e_phoff
    }

    pub fn e_shoff(&self) -> Addr {
        self.e_shoff
    }

    pub fn e_phnum(&self) -> u16 {
        self.e_phnum
    }

    pub fn e_shnum(&self) -> u16 {
        self.e_shnum
    }
}



#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;
    #[test]
    fn elf_header() {
        let bytes = include_bytes!("/home/m3m0ry/fun/osdev/assembly/hello");
        let mut reader = Reader::from_bytes(bytes);
        let elf_header = ElfHeader::parse(&mut reader).unwrap();
        assert_eq!(elf_header.e_type, FileType::EtExec);
        assert_eq!(elf_header.e_machine, Machine::AmdX86_64);
        assert_eq!(Addr(0x00401000), elf_header.e_entry);
    }

    #[test]
    fn elf() {
        let bytes = fs::read("/home/m3m0ry/fun/osdev/assembly/hello").unwrap();
        let elf = Elf64::parse(&bytes).unwrap();
        println!("{:?}", elf);
    }
}
