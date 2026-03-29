use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_lslocks::run(rustutils_lslocks::Args::parse())
}
