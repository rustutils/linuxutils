use clap::Parser;
use linuxutils_system::setsid::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
