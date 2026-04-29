use clap::Parser;
use linuxutils_system::pivot_root::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
