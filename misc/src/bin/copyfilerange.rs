use clap::Parser;
use linuxutils_misc::copyfilerange::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
