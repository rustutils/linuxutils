use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_isosize::run(rustutils_isosize::Args::parse())
}
