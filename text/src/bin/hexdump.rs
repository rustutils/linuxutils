use clap::Parser;
use linuxutils_text::hexdump::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
