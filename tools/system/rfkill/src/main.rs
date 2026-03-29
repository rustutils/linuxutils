use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_rfkill::run(rustutils_rfkill::Args::parse())
}
