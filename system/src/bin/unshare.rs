use clap::Parser;
use linuxutils_system::unshare::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
