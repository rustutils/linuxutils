use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_rename::run(rustutils_rename::Args::parse())
}
