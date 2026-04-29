use clap::Parser;
use linuxutils_system::flock::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
