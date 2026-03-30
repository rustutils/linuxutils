use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_chrt::run(rustutils_chrt::Args::parse())
}
