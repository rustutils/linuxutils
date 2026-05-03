use clap::Parser;
use linuxutils_system::setarch::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
