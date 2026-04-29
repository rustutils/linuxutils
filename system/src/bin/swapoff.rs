use clap::Parser;
use linuxutils_system::swapoff::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
