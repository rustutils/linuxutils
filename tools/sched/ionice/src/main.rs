use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_ionice::run(linuxutils_ionice::Args::parse())
}
