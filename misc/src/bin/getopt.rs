use clap::Parser;
use linuxutils_misc::getopt::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
