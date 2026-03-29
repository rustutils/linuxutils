use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_mountpoint::run(rustutils_mountpoint::Args::parse())
}
