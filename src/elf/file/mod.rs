use crate::endian;

use std::io::Read;

use endian::{Reader, Writer};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

pub trait Serde<R> {
    fn from_io<T: std::io::Read + std::io::Seek>(
        order: &endian::Encoding,
        input: &mut T,
    ) -> Result<R>;
    fn to_io<T: std::io::Write>(&self, order: &endian::Encoding, output: &mut T) -> Result<usize>;
}

// our generic Elf file parsing error
#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Parse(String),
}

// Implement std::convert::From for Error; from io::Error
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IO(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            Error::IO(err) => write!(f, "IO error: {}", err),
            Error::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // Return the underlying IO error
            Error::IO(err) => Some(err),
            // There is no underlying error
            Error::Parse(_) => None,
        }
    }
}

// Define a generic alias for a `Result` with the error type `ParseIntError`.
type Result<T> = std::result::Result<T, Error>;

// The size of the e_ident array.
const EI_NIDENT: usize = 16;

// Elf the magic number.
const EI_MAG0: u8 = 0x7f;
const EI_MAG1: u8 = b'E';
const EI_MAG2: u8 = b'L';
const EI_MAG3: u8 = b'F';

#[derive(Debug, Copy, Clone)]
struct Magic([u8; 4]);
impl Magic {
    fn new() -> Magic {
        let inner: [u8; 4] = [0; 4];
        Magic(inner)
    }

    // Tests whether the magic value are correct according to the ELF header format
    fn is_valid(&self) -> bool {
        self.0 == [EI_MAG0, EI_MAG1, EI_MAG2, EI_MAG3]
    }

    const fn len() -> usize {
        4
    }
}

impl Serde<Magic> for Magic {
    fn from_io<T: std::io::Read + std::io::Seek>(
        _: &endian::Encoding,
        input: &mut T,
    ) -> Result<Magic> {
        let mut magic = Magic::new();
        input.read_exact(&mut magic.0)?;

        if !magic.is_valid() {
            return Err(Error::Parse(String::from(
                "Invalid ELF header, incorrect magic values",
            )));
        }
        Ok(magic)
    }

    fn to_io<T: std::io::Write>(&self, _: &endian::Encoding, output: &mut T) -> Result<usize> {
        Ok(output.write(&self.0)?)
    }
}

type IdentRemaining = [u8; Identification::len() - Magic::len() - 2];

#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum AddressFormat {
    None,
    ThirtyTwoBit,
    SixtyFourBit,
}

#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum Encoding {
    None,
    LittleEndian,
    BigEndian,
}

// the identifier for the header file format
// This array of bytes specifies to interpret the file, independent
// of the processor or the file's remaining contents.
#[derive(Debug)]
pub struct Identification {
    // EI_MAGX: the magic number
    magic: Magic,
    // EI_CLASS: The fifth byte identifies the architecture
    pub class: AddressFormat,
    // EI_DATA: The sixth byte specifies the data encoding of the processor-specific data in the file.
    data: Encoding,
    remaining: IdentRemaining,
}

impl Identification {
    fn new(magic: Magic, class: AddressFormat, data: Encoding) -> Identification {
        Identification {
            magic,
            class,
            data,
            remaining: [0; Identification::len() - Magic::len() - 2],
        }
    }

    const fn len() -> usize {
        EI_NIDENT
    }
}

impl Serde<Identification> for Identification {
    fn from_io<T: std::io::Read + std::io::Seek>(
        order: &endian::Encoding,
        input: &mut T,
    ) -> Result<Identification> {
        let magic = Magic::from_io(order, input)?;

        let mut bytes = input.bytes();
        let class = bytes
            .next()
            .and_then(|result| result.ok())
            .and_then(|byte| AddressFormat::from_u8(byte))
            .ok_or(Error::Parse(String::from("Could not read class type")))?;

        if class == AddressFormat::None {
            return Err(Error::Parse(format!("Invalid address format: {:?}", class)));
        }

        let data = bytes
            .next()
            .and_then(|result| result.ok())
            .and_then(|byte| Encoding::from_u8(byte))
            .ok_or(Error::Parse(String::from("Could not read encoding type")))?;

        if data == Encoding::None {
            return Err(Error::Parse(format!("Invalid encoding: {:?}", data)));
        }

        let mut identification = Identification::new(magic, class, data);

        input.read_exact(&mut identification.remaining)?;
        Ok(identification)
    }

    fn to_io<T: std::io::Write>(&self, order: &endian::Encoding, output: &mut T) -> Result<usize> {
        let mut written = self.magic.to_io(order, output)?;

        let class: u8 = self
            .class
            .to_u8()
            .ok_or(Error::Parse(String::from("Could not convert class")))?;
        written += output.write(&[class])?;

        let data: u8 = self
            .data
            .to_u8()
            .ok_or(Error::Parse(String::from("Could not convert data")))?;
        written += output.write(&[data])?;

        written += output.write(&self.remaining)?;

        Ok(written)
    }
}

#[repr(u16)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Type {
    None,
    Relocatable,
    Executable,
    Dynamic,
    Core,
}

#[repr(u16)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Architecture {
    None = 0,
    M32 = 1,
    SPARC = 2,
    I386 = 3,
    M68K = 4,
    M88K = 5,
    I860 = 7,
    MIPS = 8,
    PARISC = 9,
    SPARC32PLUS = 18,
    PPC = 20,
    PPC64 = 21,
    S390 = 22,
    ARM = 40,
    SH = 42,
    SPARCV9 = 43,
    IA_64 = 50,
    X86_64 = 62,
    VAX = 75,
}

#[repr(u32)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Version {
    None,
    Current,
}

#[derive(Debug)]
pub struct Header {
    ident: Identification,
    // This member of the structure identifies the object file type
    e_type: Type,
    //  This member specifies the required architecture for an individual file
    machine: Architecture,
    version: Version,
    // This member gives the virtual address to which the system
    // first transfers control, thus starting the process.  If the
    // file has no associated entry point, this member holds zero.
    // Note: We up-size this to u64 to simplify the type system, but it will serialize according
    //       to the address format
    entry: u64,
}

impl Header {
    fn new(
        ident: Identification,
        e_type: Type,
        machine: Architecture,
        version: Version,
        entry: u64,
    ) -> Header {
        Header {
            ident,
            e_type,
            machine,
            version,
            entry,
        }
    }
}

impl Serde<Header> for Header {
    fn from_io<T: std::io::Read + std::io::Seek>(
        order: &endian::Encoding,
        input: &mut T,
    ) -> Result<Header> {
        let ident = Identification::from_io(order, input)?;
        let endian = match ident.data {
            Encoding::LittleEndian => endian::Encoding::Little,
            Encoding::BigEndian => endian::Encoding::Big,
            _ => panic!("should not be hit."),
        };

        let e_type = Type::from_u16(endian.read_u16(input)?)
            .ok_or(Error::Parse(String::from("Could not read type")))?;

        let machine = Architecture::from_u16(endian.read_u16(input)?)
            .ok_or(Error::Parse(String::from("Could not read machine")))?;

        let version = Version::from_u32(endian.read_u32(input)?)
            .ok_or(Error::Parse(String::from("Could not read version")))?;

        let entry = match ident.class {
            AddressFormat::None => return Err(Error::Parse(String::from("Could not read version"))),
            AddressFormat::ThirtyTwoBit => endian.read_u32(input).map(|v| v as u64),
            AddressFormat::SixtyFourBit => endian.read_u64(input),
        }?;

        Ok(Header {
            ident,
            e_type,
            machine,
            version,
            entry,
        })
    }

    fn to_io<T: std::io::Write>(&self, order: &endian::Encoding, output: &mut T) -> Result<usize> {
        let mut written = self.ident.to_io(order, output)?;

        let endian = match self.ident.data {
            Encoding::LittleEndian => endian::Encoding::Little,
            Encoding::BigEndian => endian::Encoding::Big,
            _ => panic!("should not be hit."),
        };

        let e_type = self
            .e_type
            .to_u16()
            .ok_or(Error::Parse(String::from("Could not convert type")))?;

        let machine = self
            .machine
            .to_u16()
            .ok_or(Error::Parse(String::from("Could not convert machine")))?;

        let version = self
            .version
            .to_u32()
            .ok_or(Error::Parse(String::from("Could not convert version")))?;

        endian.write_u16(e_type, output)?;
        endian.write_u16(machine, output)?;
        endian.write_u32(version, output)?;

        written += match self.ident.class {
            AddressFormat::None => return Err(Error::Parse(String::from("Could not write version"))),
            AddressFormat::ThirtyTwoBit => {
                endian.write_u32(self.entry as u32, output).map(|_| 4)
            }
            AddressFormat::SixtyFourBit => {
                endian.write_u64(self.entry, output).map(|_| 8)
            }
        }?;

        Ok(written + 2 + 2 + 4)
    }
}

pub struct File {
    header: Header,
    remaining: Vec<u8>,
}

impl File {
    fn new(header: Header) -> File {
        File {
            header,
            remaining: vec![0; 0],
        }
    }
}

impl Serde<File> for File {
    fn from_io<T: std::io::Read + std::io::Seek>(
        order: &endian::Encoding,
        input: &mut T,
    ) -> Result<File> {
        let header = Header::from_io(order, input)?;
        let mut file = File::new(header);
        input.read_to_end(&mut file.remaining)?;
        Ok(file)
    }

    fn to_io<T: std::io::Write>(&self, order: &endian::Encoding, output: &mut T) -> Result<usize> {
        let amount = self.header.to_io(order, output)? + output.write(&self.remaining)?;
        Ok(amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod magic {
        use super::*;
        #[test]
        fn from_io_valid() -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut valid = std::io::Cursor::new([EI_MAG0, EI_MAG1, EI_MAG2, EI_MAG3]);
            Magic::from_io(&endian::Encoding::Any, &mut valid)?;
            Ok(())
        }

        #[test]
        fn from_io_invalid() {
            let mut valid = std::io::Cursor::new("not at all invalid");
            let result = Magic::from_io(&endian::Encoding::Any, &mut valid);
            assert!(result.is_err(), "expected error for invalid input");
        }
    }

    mod file {
        use super::*;

        #[test]
        fn round_trip() -> std::result::Result<(), Box<dyn std::error::Error>> {
            let expected = include_bytes!("hello_world.o").to_vec();
            let mut input = std::io::Cursor::new(&expected);
            let parsed = File::from_io(&endian::Encoding::Any, &mut input)?;

            let mut actual: Vec<u8> = vec![0; 0];
            let mut output = std::io::Cursor::new(&mut actual);
            let written = parsed.to_io(&endian::Encoding::Any, &mut output)?;

            assert_eq!(expected.len(), written);
            assert_eq!(expected, actual);
            Ok(())
        }
    }
}
