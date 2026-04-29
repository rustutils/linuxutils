use clap::Parser;
use linuxutils_text::line::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
