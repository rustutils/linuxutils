use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_setsid::run(rustutils_setsid::Args::parse())
}
