use clap::Parser;
use linuxutils_text::column::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
