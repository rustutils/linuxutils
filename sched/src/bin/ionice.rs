use clap::Parser;
use linuxutils_sched::ionice::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
