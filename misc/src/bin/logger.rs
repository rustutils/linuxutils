use clap::Parser;
use linuxutils_misc::logger::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
