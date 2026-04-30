use clap::Parser;
use linuxutils_disk::umount::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
