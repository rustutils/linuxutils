use clap::Parser;
use linuxutils_misc::fincore::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
