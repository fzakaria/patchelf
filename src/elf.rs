///
/// This is a simple elf library for reading and writing to the ELF header
/// @see https://linux.die.net/man/5/elf

pub mod file {
    use std::io::{Seek, Read};

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
            match *self {
                // Both underlying errors already impl `Display`, so we defer to
                // their implementations.
                Error::IO(ref err) => write!(f, "IO error: {}", err),
                Error::Parse(ref err) => write!(f, "Parse error: {}", err),
            }
        }
    }

    impl std::error::Error for Error {
        fn description(&self) -> &str {
            // Both underlying errors already impl `Error`, so we defer to their
            // implementations.
            match *self {
                Error::IO(ref err) => err.description(),
                // Normally we can just write `err.description()`, but the error
                // type has a concrete method called `description`, which conflicts
                // with the trait method. For now, we must explicitly call
                // `description` through the `Error` trait.
                Error::Parse(ref err) => &err,
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
            self.0[0] == EI_MAG0 && self.0[1] == EI_MAG1 && self.0[2] == EI_MAG2 && self.0[3] == EI_MAG3
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

    impl File {
        pub fn from_io<T: std::io::Read + std::io::Seek>(input: &mut T) -> Result<File> {
            let header = Header::from_io(input)?;

            let mut remaining = vec![0;0];
            input.read_to_end(&mut remaining)?;
            Ok(File {
                header,
                remaining
            })
        }
    }
}
