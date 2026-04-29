use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_sched::ionice::run(linuxutils_sched::ionice::Args::parse())
}
