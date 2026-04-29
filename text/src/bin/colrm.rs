use clap::Parser;
use linuxutils_text::colrm::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
