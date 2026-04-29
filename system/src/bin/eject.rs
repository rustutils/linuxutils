use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::eject::run(linuxutils_system::eject::Args::parse())
}
