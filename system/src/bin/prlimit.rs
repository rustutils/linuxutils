use clap::Parser;
use linuxutils_system::prlimit::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
