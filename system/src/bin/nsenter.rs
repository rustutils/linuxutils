use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::nsenter::run(linuxutils_system::nsenter::Args::parse())
}
