use std::{fmt, ops::Range};

pub mod addr;
pub mod error;
pub mod file_type;
pub mod machine;
pub mod segment;
pub mod reader;

use crate::{
    addr::Addr,
    error::{ElfError, ElfHeaderError, ProgramHeaderError},
    file_type::FileType,
    machine::Machine,
    segment::{SegmentType, SegmentFlags},
    reader::Reader,
};

/// Structure that represents an Elf 64-bit file
/// We are only parsing x86 ISA little endian Elfs
pub struct Elf64 {
    pub elf_header: ElfHeader,
    /// Program Header table
    pub ph_table: Vec<ProgramHeader>,
}

impl Elf64 {
    pub fn parse(bytes: &[u8]) -> Result<Self, ElfError> {
        let mut reader = Reader::from_bytes(bytes);
        let elf_header = ElfHeader::parse(&mut reader)?;

        // Allocate a new vector to hold the Program header table
        let mut ph_table = Vec::with_capacity(elf_header.e_phnum().into());

        // Move the read cursor to the program header table beginning
        reader.seek(elf_header.e_phoff().into())?;

        for index in 0..elf_header.e_phnum() {
            ph_table.push(ProgramHeader::parse(&mut reader)?);
        }

        Ok(Self {
            elf_header,
            ph_table,
        })
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
#[derive(Debug)]
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
    data: Vec<u8>,
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

        Ok(Self {
            p_type,
            p_flags,
            p_offset,
            p_vaddr,
            p_paddr,
            p_filesz,
            p_memsz,
            p_align,
            data
        })
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


        Ok(ElfHeader{
            e_type,
            e_machine,
            e_entry,
            e_phoff,
            e_shoff,
            e_phentsize,
            e_phnum,
        })
    }

    pub fn e_phoff(&self) -> Addr {
        self.e_phoff
    }

    pub fn e_phnum(&self) -> u16 {
        self.e_phnum
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
