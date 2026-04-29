use clap::Parser;
use linuxutils_system::choom::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
