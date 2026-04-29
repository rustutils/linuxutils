use clap::Parser;
use linuxutils_system::mountpoint::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
