use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_ionice::run(rustutils_ionice::Args::parse())
}
