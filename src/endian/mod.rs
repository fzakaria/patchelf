use std::io::{Read, Result, Seek};

#[derive(Debug)]
pub enum Reader {
    Little,
    Big,
}

impl Reader {

    /// Read a primitive value with this endianness from the given source.
    pub fn read_u16<S>(&self, src: &mut S) -> Result<u16>
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

    /// Read a primitive value with this endianness from the given source.
    pub fn read_u32<S>(&self, src: &mut S) -> Result<u32>
        where
            S: Read,
    {
        let mut buf = [0; 4];
        src.read_exact(&mut buf)?;

        Ok(match *self {
            Reader::Little => u32::from_le_bytes(buf),
            Reader::Big => u32::from_be_bytes(buf),
        })
    }

    /// Read a primitive value with this endianness from the given source.
    pub fn read_u64<S>(&self, src: &mut S) -> Result<u64>
        where
            S: Read,
    {
        let mut buf = [0; 8];
        src.read_exact(&mut buf)?;

        Ok(match *self {
            Reader::Little => u64::from_le_bytes(buf),
            Reader::Big => u64::from_be_bytes(buf),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_conversion() {
        let expected :u16 = 1337;
        let bytes = expected.to_be_bytes();
    }
}