use clap::Parser;
use linuxutils_disk::mount::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
