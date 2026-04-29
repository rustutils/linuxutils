use clap::Parser;
use linuxutils_system::lscpu::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
