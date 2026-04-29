use clap::Parser;
use linuxutils_system::eject::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
