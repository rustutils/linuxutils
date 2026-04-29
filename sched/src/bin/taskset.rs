use clap::Parser;
use linuxutils_sched::taskset::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
