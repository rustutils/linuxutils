use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::renice::run(linuxutils_system::renice::Args::parse())
}
