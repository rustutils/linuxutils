use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_rev::run(rustutils_rev::Args::parse())
}
