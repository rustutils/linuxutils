use clap::Parser;
use linuxutils_misc::namei::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
