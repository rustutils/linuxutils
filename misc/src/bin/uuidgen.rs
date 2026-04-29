use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::uuidgen::run(linuxutils_misc::uuidgen::Args::parse())
}
