use clap::Parser;
use linuxutils_misc::kill::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
