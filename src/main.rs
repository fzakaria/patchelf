#[macro_use]
extern crate num_derive;
extern crate getopts;
extern crate byteorder;

mod elf;
mod endian;

use elf::file::Serde;
use elf::file::File;

use getopts::Options;
use std::env;
use std::path::Path;
use byteorder::LittleEndian;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILENAME [options]", program);
    print!("{}", opts.usage(&brief));
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

    // the endian at the start of the file read does not matter; so just set it to native endian
    let elf: File<u32> = elf::file::File::from_io(&encoding, &mut binary)?;

    let mut output = std::fs::File::create("/tmp/output")?;

    elf.to_io(&encoding, &mut output)?;

    Ok(())
}
