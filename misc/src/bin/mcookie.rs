use clap::Parser;
use linuxutils_misc::mcookie::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
