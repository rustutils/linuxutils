use clap::Parser;
use linuxutils_text::rev::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
