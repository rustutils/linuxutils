use clap::Parser;
use linuxutils_misc::waitpid::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
