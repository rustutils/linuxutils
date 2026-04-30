use clap::Parser;
use linuxutils_misc::blkid::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
