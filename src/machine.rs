#[derive(Debug, PartialEq)]
pub enum Machine {
    X86 = 0x03,
    AmdX86_64 = 0x3E,
}

impl TryFrom<u16> for Machine {
    type Error = Error;
    fn try_from(value: u16) -> Result<Machine, Self::Error> {
        match value {
            0x03 => Ok(Machine::X86),
            0x3E => Ok(Machine::AmdX86_64),
            _ => Err(Error::NotSupported),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    NotSupported
}
