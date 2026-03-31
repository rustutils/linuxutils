use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_uuidparse::run(linuxutils_uuidparse::Args::parse())
}
