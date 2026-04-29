use clap::Parser;
use linuxutils_system::lsns::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
