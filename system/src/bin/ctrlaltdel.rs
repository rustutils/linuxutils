use clap::Parser;
use linuxutils_system::ctrlaltdel::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
