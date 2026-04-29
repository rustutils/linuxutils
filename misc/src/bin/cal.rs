use clap::Parser;
use linuxutils_misc::cal::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
