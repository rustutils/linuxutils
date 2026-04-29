use clap::Parser;
use linuxutils_misc::uuidparse::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
