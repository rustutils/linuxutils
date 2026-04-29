use clap::Parser;
use linuxutils_misc::uuidgen::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
