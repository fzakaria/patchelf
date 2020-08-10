use std::io::{Read, Result, Seek, Write};
use byteorder::{ByteOrder, WriteBytesExt, ReadBytesExt, LittleEndian, BigEndian, NativeEndian};

pub enum Encoding {
    Little,
    Big,
    Any
}

pub trait Reader {
    fn read_u16<S>(&self, src: &mut S) -> Result<u16> where S: Read;
    fn read_u32<S>(&self, src: &mut S) -> Result<u32> where S: Read;
    fn read_u64<S>(&self, src: &mut S) -> Result<u64> where S: Read;
}

// This trait allows ByteOrder themselves to be dynamically
pub trait Writer {
    fn write_u8<S>(&self, value: u8, target: &mut S) -> Result<()> where S: Write;
    fn write_u16<S>(&self, value: u16, target: &mut S) -> Result<()> where S: Write;
    fn write_u32<S>(&self, value: u32, target: &mut S) -> Result<()> where S: Write;

}

impl Writer for Encoding {

    fn write_u8<S>(&self, value: u8, target: &mut S) -> Result<()>
        where S: Write {
        target.write_u8(value)
    }

    fn write_u16<S>(&self, value: u16, target: &mut S) -> Result<()>
        where S: Write,
    {
        match *self {
            Encoding::Little => target.write_u16::<LittleEndian>(value),
            Encoding::Big => target.write_u16::<BigEndian>(value),
            Encoding::Any => target.write_u16::<NativeEndian>(value),
        }
    }

    fn write_u32<S>(&self, value: u32, target: &mut S) -> Result<()>
        where S: Write,
    {
        match *self {
            Encoding::Little => target.write_u32::<LittleEndian>(value),
            Encoding::Big => target.write_u32::<BigEndian>(value),
            Encoding::Any => target.write_u32::<NativeEndian>(value),
        }
    }
}

impl Reader for Encoding {

    /// Read a primitive value with this endianness from the given source.
    fn read_u16<S>(&self, src: &mut S) -> Result<u16>
    where
        S: Read,
    {
        match *self {
            Encoding::Little => src.read_u16::<LittleEndian>(),
            Encoding::Big => src.read_u16::<BigEndian>(),
            Encoding::Any => src.read_u16::<NativeEndian>(),
        }
    }

    /// Read a primitive value with this endianness from the given source.
    fn read_u32<S>(&self, src: &mut S) -> Result<u32>
        where
            S: Read,
    {
        match *self {
            Encoding::Little => src.read_u32::<LittleEndian>(),
            Encoding::Big => src.read_u32::<BigEndian>(),
            Encoding::Any => src.read_u32::<NativeEndian>(),
        }
    }

    /// Read a primitive value with this endianness from the given source.
    fn read_u64<S>(&self, src: &mut S) -> Result<u64>
        where
            S: Read,
    {
        match *self {
            Encoding::Little => src.read_u64::<LittleEndian>(),
            Encoding::Big => src.read_u64::<BigEndian>(),
            Encoding::Any => src.read_u64::<NativeEndian>(),
        }
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