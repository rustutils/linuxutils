use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_look::run(rustutils_look::Args::parse())
}
