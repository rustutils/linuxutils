use clap::Parser;
use linuxutils_misc::exch::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
