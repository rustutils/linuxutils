use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_taskset::run(linuxutils_taskset::Args::parse())
}
