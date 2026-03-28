use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_colrm::run(rustutils_colrm::Args::parse())
}
