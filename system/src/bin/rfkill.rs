use clap::Parser;
use linuxutils_system::rfkill::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
