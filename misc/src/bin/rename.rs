use clap::Parser;
use linuxutils_misc::rename::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
