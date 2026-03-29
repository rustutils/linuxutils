use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_uuidparse::run(rustutils_uuidparse::Args::parse())
}
