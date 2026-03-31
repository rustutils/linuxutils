use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_flock::run(linuxutils_flock::Args::parse())
}
