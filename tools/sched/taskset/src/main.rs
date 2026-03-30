use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_taskset::run(rustutils_taskset::Args::parse())
}
