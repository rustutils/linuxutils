use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::flock::run(linuxutils_system::flock::Args::parse())
}
