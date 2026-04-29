use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_sched::taskset::run(linuxutils_sched::taskset::Args::parse())
}
