use clap::Parser;
use linuxutils_system::ipcrm::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
