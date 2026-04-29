use clap::Parser;
use linuxutils_system::blkdiscard::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
