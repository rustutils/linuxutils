use clap::Parser;
use linuxutils_disk::blockdev::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
