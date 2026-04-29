use clap::Parser;
use linuxutils_system::fallocate::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
