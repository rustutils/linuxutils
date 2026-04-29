use clap::Parser;
use linuxutils_system::fadvise::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
