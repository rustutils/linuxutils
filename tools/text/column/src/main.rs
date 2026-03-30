use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_column::run(rustutils_column::Args::parse())
}
