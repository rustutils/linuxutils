use clap::Parser;
use linuxutils_system::nsenter::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
