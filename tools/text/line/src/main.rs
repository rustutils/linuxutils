use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_line::run(rustutils_line::Args::parse())
}
