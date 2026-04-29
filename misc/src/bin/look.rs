use clap::Parser;
use linuxutils_misc::look::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
