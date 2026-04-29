use clap::Parser;
use linuxutils_system::renice::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
