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

fn parse_elf_file<T>(file: &mut std::fs::File) -> std::result::Result<elf::file::File<T>, Box<dyn std::error::Error>>
where T: PrimInt + FromPrimitive
{
    let encoding = endian::Encoding::Any;

    // red the header first
    let identification = elf::file::Identification::from_io(&encoding, file)?;
    // reset the file IO read
    file.seek(SeekFrom::Start(0))?;

    // the endian at the start of the file read does not matter; so just set it to native endian
    let elf: elf::file::File<T> = match identification.class {
        elf::file::AddressFormat::None => panic!("This should not happen after a successful parse"),
        elf::file::AddressFormat::ThirtyTwoBit => elf::file::File::<u32>::from_io(&encoding, file)?,
        elf::file::AddressFormat::SixtyFourBit => elf::file::File::<u64>::from_io(&encoding, file)?,
    };

    Ok(elf)
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
    let elf = parse_elf_file(&mut binary)?;

    let mut output = std::fs::File::create("/tmp/output")?;

    let encoding = endian::Encoding::Any;
    elf.to_io(&encoding, &mut output)?;

    Ok(())
}
