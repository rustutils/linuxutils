use clap::Parser;
use linuxutils_term::write::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
