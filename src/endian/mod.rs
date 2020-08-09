use std::io::{Read, Result, Seek};

#[derive(Debug)]
pub enum Reader {
    Little,
    Big,
}

impl Endian {
    /// Read a primitive value with this endianness from the given source.
    pub fn read_u16<S>(&self, mut src: S) -> Result<u16>
    where
        S: Read,
    {
        let mut buf = [0; 2];
        src.read_exact(&mut buf)?;

        Ok(match *self {
            Reader::Little => u16::from_le_bytes(buf),
            Reader::Big => u16::from_be_bytes(buf),
        })
    }
}
