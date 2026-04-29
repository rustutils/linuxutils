use clap::Parser;
use linuxutils_system::setpgid::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
