use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_uuidgen::run(linuxutils_uuidgen::Args::parse())
}
