use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::lsns::run(linuxutils_system::lsns::Args::parse())
}
