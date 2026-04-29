use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_sched::chrt::run(linuxutils_sched::chrt::Args::parse())
}
