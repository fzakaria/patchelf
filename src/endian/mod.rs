use std::io::{Read, Result, Seek, Write};

pub trait Reader {
    fn read_u16<S>(&self, src: &mut S) -> Result<u16> where S: Read;
    fn read_u32<S>(&self, src: &mut S) -> Result<u32> where S: Read;
    fn read_u64<S>(&self, src: &mut S) -> Result<u64> where S: Read;
}

pub trait Writer {
    fn write_u8<S>(&self, value: u8, target: &mut S) -> Result<()> where S: Write;
    fn write_u16<S>(&self, value: u16, target: &mut S) -> Result<()> where S: Write;
    fn write_u32<S>(&self, value: u32, target: &mut S) -> Result<()> where S: Write;

}


#[derive(Debug)]
pub enum Encoding {
    Little,
    Big,
}

impl Writer for Encoding {

    fn write_u8<S>(&self, value: u8, target: &mut S) -> Result<()>
        where S: Write {
        target.write_all(&[value])
    }

    fn write_u16<S>(&self, value: u16, target: &mut S) -> Result<()>
        where S: Write,
    {
        let bytes = match *self {
            Encoding::Little => value.to_le_bytes(),
            Encoding::Big => value.to_be_bytes(),
        };

        target.write_all(&bytes)
    }

    fn write_u32<S>(&self, value: u32, target: &mut S) -> Result<()>
        where S: Write,
    {
        let bytes = match *self {
            Encoding::Little => value.to_le_bytes(),
            Encoding::Big => value.to_be_bytes(),
        };

        target.write_all(&bytes)
    }
}

impl Reader for Encoding {

    /// Read a primitive value with this endianness from the given source.
    fn read_u16<S>(&self, src: &mut S) -> Result<u16>
    where
        S: Read,
    {
        let mut buf = [0; 2];
        src.read_exact(&mut buf)?;

        Ok(match *self {
            Encoding::Little => u16::from_le_bytes(buf),
            Encoding::Big => u16::from_be_bytes(buf),
        })
    }

    /// Read a primitive value with this endianness from the given source.
    fn read_u32<S>(&self, src: &mut S) -> Result<u32>
        where
            S: Read,
    {
        let mut buf = [0; 4];
        src.read_exact(&mut buf)?;

        Ok(match *self {
            Encoding::Little => u32::from_le_bytes(buf),
            Encoding::Big => u32::from_be_bytes(buf),
        })
    }

    /// Read a primitive value with this endianness from the given source.
    fn read_u64<S>(&self, src: &mut S) -> Result<u64>
        where
            S: Read,
    {
        let mut buf = [0; 8];
        src.read_exact(&mut buf)?;

        Ok(match *self {
            Encoding::Little => u64::from_le_bytes(buf),
            Encoding::Big => u64::from_be_bytes(buf),
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