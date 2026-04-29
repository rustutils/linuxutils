use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::fsfreeze::run(linuxutils_system::fsfreeze::Args::parse())
}
