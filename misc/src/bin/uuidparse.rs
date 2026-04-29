use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::uuidparse::run(linuxutils_misc::uuidparse::Args::parse())
}
