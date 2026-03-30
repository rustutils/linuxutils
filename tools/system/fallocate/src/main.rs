use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_fallocate::run(rustutils_fallocate::Args::parse())
}
