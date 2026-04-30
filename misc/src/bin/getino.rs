use clap::Parser;
use linuxutils_misc::getino::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
