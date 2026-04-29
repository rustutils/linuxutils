use clap::Parser;
use linuxutils_misc::lslocks::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
