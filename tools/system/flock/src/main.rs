use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_flock::run(rustutils_flock::Args::parse())
}
