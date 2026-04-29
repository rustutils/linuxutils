use clap::Parser;
use linuxutils_sched::chrt::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
