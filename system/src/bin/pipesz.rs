use clap::Parser;
use linuxutils_system::pipesz::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
