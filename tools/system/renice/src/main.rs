use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_renice::run(rustutils_renice::Args::parse())
}
