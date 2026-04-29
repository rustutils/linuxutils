use clap::Parser;
use linuxutils_system::fsfreeze::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
