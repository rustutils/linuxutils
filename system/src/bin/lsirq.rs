use clap::Parser;
use linuxutils_system::lsirq::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
