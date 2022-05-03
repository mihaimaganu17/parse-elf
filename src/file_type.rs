#[derive(Debug, PartialEq)]
pub enum FileType {
    EtNone = 0x0,
    EtRel = 0x1,
    EtExec = 0x2,
    EtDyn = 0x3,
    EtCore = 0x4,
}

impl TryFrom<u16> for FileType {
    type Error = Error;
    fn try_from(value: u16) -> Result<FileType, Self::Error> {
        match value {
            0x0 => Ok(FileType::EtNone),
            0x1 => Ok(FileType::EtRel),
            0x2 => Ok(FileType::EtExec),
            0x3 => Ok(FileType::EtDyn),
            0x4 => Ok(FileType::EtCore),
            _ => Err(Error::Unsupported),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Unsupported,
}
