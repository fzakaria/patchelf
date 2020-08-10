#[macro_use]
extern crate num_derive;
extern crate getopts;
extern crate byteorder;

mod elf;
mod endian;

use elf::file::Serde;
use elf::file::Identification;

use getopts::Options;
use std::env;
use std::path::Path;
use byteorder::LittleEndian;
use std::io::{self, Seek, SeekFrom, Read};
use num_traits::{PrimInt, FromPrimitive};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILENAME [options]", program);
    print!("{}", opts.usage(&brief));
}

fn handle_elf_file<T>(file: elf::file::File<T>) -> std::result::Result<(), Box<dyn std::error::Error>>
where T: PrimInt + FromPrimitive {
    let encoding = endian::Encoding::Any;

    let mut output = std::fs::File::create("/tmp/output")?;

    file.to_io(&encoding, &mut output)?;

    Ok(())
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program = Path::new(&args[0])
        .file_name()
        .and_then(|str| str.to_str())
        .expect("should never fail.");

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "print-rpath", "prints the RPATH of the elf binary");
    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    if matches.free.is_empty() {
        print_usage(&program, opts);
        return Ok(());
    }

    let path = &matches.free[0];

    let mut binary = std::fs::File::open(path)?;

    let encoding = endian::Encoding::Any;

    // red the header first
    let identification = elf::file::Identification::from_io(&encoding, &mut binary)?;
    // reset the file IO read
    binary.seek(SeekFrom::Start(0))?;

    match identification.class {
        elf::file::AddressFormat::None => panic!("This should not happen after a successful parse"),
        elf::file::AddressFormat::ThirtyTwoBit => handle_elf_file(elf::file::File::<u32>::from_io(&encoding, &mut binary)?)?,
        elf::file::AddressFormat::SixtyFourBit => handle_elf_file(elf::file::File::<u32>::from_io(&encoding, &mut binary)?)?,
    };

    Ok(())
}
