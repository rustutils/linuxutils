use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_mcookie::run(rustutils_mcookie::Args::parse())
}
