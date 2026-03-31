use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_mountpoint::run(linuxutils_mountpoint::Args::parse())
}
