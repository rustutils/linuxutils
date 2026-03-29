use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_prlimit::run(rustutils_prlimit::Args::parse())
}
