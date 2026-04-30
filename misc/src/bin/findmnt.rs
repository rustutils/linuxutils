use clap::Parser;
use linuxutils_misc::findmnt::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
