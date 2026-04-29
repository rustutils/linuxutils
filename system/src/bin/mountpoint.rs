use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::mountpoint::run(linuxutils_system::mountpoint::Args::parse())
}
