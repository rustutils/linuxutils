use clap::Parser;
use linuxutils_term::wall::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
