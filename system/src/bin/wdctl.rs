use clap::Parser;
use linuxutils_system::wdctl::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
