pub trait Serde<R> {
    fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<R>;
    fn to_io<T: std::io::Write>(&self, output: &mut T) -> Result<usize>;
}

// our generic Elf file parsing error
#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Parse(String)
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
static EI_NIDENT: usize = 16;

// Elf the magic number.
static EI_MAG0: u8 = 0x7f;
static EI_MAG1: u8 = b'E';
static EI_MAG2: u8 = b'L';
static EI_MAG3: u8 = b'F';

#[derive(Debug, Copy, Clone)]
struct Magic([u8; 4]);
impl Magic {
    // Tests whether the magic value are correct according to the ELF header format
    fn is_valid(&self) -> bool {
        self.0 == [EI_MAG0, EI_MAG1, EI_MAG2, EI_MAG3]
    }

    pub fn from_io<T: std::io::Read + std::io::Seek>(file: &mut T) -> Result<Magic> {
        // seek to the start of the file always
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut magic: [u8; 4] = [0; 4];
        file.read_exact(&mut magic)?;
        let magic = Magic(magic);
        if !magic.is_valid() {
            return Err(Error::Parse(String::from("Invalid ELF header, incorrect magic values")));
        }
        Ok(magic)
    }
}

// the identifier for the header file format
// This array of bytes specifies to interpret the file, independent
// of the processor or the file's remaining contents.
#[derive(Debug, Copy, Clone)]
struct Identification {
    magic: Magic,
}

impl Identification {

    pub fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<Identification> {
        let magic = Magic::from_io(input)?;
        Ok(Identification {
            magic
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Header {
    ident: Identification,
}

impl Header {
    pub fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<Header> {
        let ident = Identification::from_io(input)?;
        Ok(Header {
            ident
        })
    }
}

#[derive(Debug, Clone)]
pub struct File {
    header: Header,
    remaining: Vec<u8>
}

impl Serde<File> for File {
    fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<File> {
        let header = Header::from_io(input)?;

        let mut remaining = vec![0;0];
        input.read_to_end(&mut remaining)?;
        Ok(File {
            header,
            remaining
        })
    }

    fn to_io<T: std::io::Write>(&self, output: &mut T) -> Result<usize> {
        let amount = output.write(&self.header.ident.magic.0)? + output.write(&self.remaining)?;
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
