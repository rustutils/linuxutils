use clap::Parser;
use linuxutils_text::bits::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
