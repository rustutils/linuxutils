use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_uuidgen::run(rustutils_uuidgen::Args::parse())
}
