use clap::Parser;
use linuxutils_system::lsmem::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
