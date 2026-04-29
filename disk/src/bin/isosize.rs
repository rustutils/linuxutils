use clap::Parser;
use linuxutils_disk::isosize::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
