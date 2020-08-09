use crate::endian;

use std::io::Read;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive, PrimInt};

pub trait Serde<R> {
    fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<R>;
    fn to_io<T: std::io::Write>(&self, output: &mut T) -> Result<usize>;
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
    fn from_io<T: std::io::Read + std::io::Seek>(file: &mut T) -> Result<Magic> {
        let mut magic = Magic::new();
        file.read_exact(&mut magic.0)?;

        if !magic.is_valid() {
            return Err(Error::Parse(String::from(
                "Invalid ELF header, incorrect magic values",
            )));
        }
        Ok(magic)
    }

    fn to_io<T: std::io::Write>(&self, output: &mut T) -> Result<usize> {
        Ok(output.write(&self.0)?)
    }
}

type IdentRemaining = [u8; Identification::len() - Magic::len() - 2];

#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum AddressFormat {
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
struct Identification {
    // EI_MAGX: the magic number
    magic: Magic,
    // EI_CLASS: The fifth byte identifies the architecture
    class: AddressFormat,
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
    fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<Identification> {
        let magic = Magic::from_io(input)?;

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

    fn to_io<T: std::io::Write>(&self, output: &mut T) -> Result<usize> {
        Ok(self.magic.to_io(output)? + output.write(&self.remaining)?)
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


trait Pointer {

}

impl Pointer for u32 {

}

impl Pointer for u64 {

}

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
    entry: Box<dyn Pointer>,
}

impl std::fmt::Debug for Header {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        // TODO: fill me in
        write!(fmt, "Header {{ }}")
    }
}

impl Header {
    fn new(
        ident: Identification,
        e_type: Type,
        machine: Architecture,
        version: Version,
        entry: Box<dyn Pointer>,
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
    fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<Header> {
        let ident = Identification::from_io(input)?;
        let endian = match ident.data {
            Encoding::LittleEndian => endian::Reader::Little,
            Encoding::BigEndian => endian::Reader::Big,
            _ => panic!("should not be hit."),
        };

        let e_type = Type::from_u16(endian.read_u16(input)?)
            .ok_or(Error::Parse(String::from("Could not read type")))?;

        let machine = Architecture::from_u16(endian.read_u16(input)?)
            .ok_or(Error::Parse(String::from("Could not read machine")))?;

        let version = Version::from_u32(endian.read_u32(input)?)
            .ok_or(Error::Parse(String::from("Could not read version")))?;

        match ident.class {
            AddressFormat::ThirtyTwoBit => {
                let entry = Box::new(endian.read_u32(input)?);
                return Ok(
                    Header { ident, e_type, machine, version, entry }
                )
            },
            AddressFormat::SixtyFourBit => {
                let entry = Box::new(endian.read_u64(input)?);
                return Ok(
                    Header { ident, e_type, machine, version, entry}
                )
            },
            _ => panic!("should not be hit."),
        };
    }

    fn to_io<T: std::io::Write>(&self, output: &mut T) -> Result<usize> {
        self.ident.to_io(output)?;

        let endian = match ident.data {
            Encoding::LittleEndian => endian::Reader::Little,
            Encoding::BigEndian => endian::Reader::Big,
            _ => panic!("should not be hit."),
        };

        let e_type =
            self.e_type.to_u8().ok_or(Error::Parse(String::from("Could not convert type")))?;
        e_type.
        Ok(0)
    }
}

pub struct File {
    header: Header,
    remaining: Vec<u8>,
}

impl File{
    fn new(header: Header) -> File {
        File {
            header,
            remaining: vec![0; 0],
        }
    }
}

impl Serde<File> for File {
    fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<File> {
        let header = Header::from_io(input)?;
        let mut file = File::new(header);
        input.read_to_end(&mut file.remaining)?;
        Ok(file)
    }

    fn to_io<T: std::io::Write>(&self, output: &mut T) -> Result<usize> {
        let amount = self.header.to_io(output)? + output.write(&self.remaining)?;
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
            Magic::from_io(&mut valid)?;
            Ok(())
        }

        #[test]
        fn from_io_invalid() {
            let mut valid = std::io::Cursor::new("not at all invalid");
            let result = Magic::from_io(&mut valid);
            assert!(result.is_err(), "expected error for invalid input");
        }
    }

    mod file {
        use super::*;

        #[test]
        fn round_trip() -> std::result::Result<(), Box<dyn std::error::Error>> {
            let expected = include_bytes!("hello_world.o").to_vec();
            let mut input = std::io::Cursor::new(&expected);
            let parsed = File::from_io(&mut input)?;

            let mut actual: Vec<u8> = vec![0; 0];
            let mut output = std::io::Cursor::new(&mut actual);
            let written = parsed.to_io(&mut output)?;

            assert_eq!(expected.len(), written);
            assert_eq!(expected, actual);
            Ok(())
        }
    }
}
