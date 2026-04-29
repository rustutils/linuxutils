use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::prlimit::run(linuxutils_system::prlimit::Args::parse())
}
