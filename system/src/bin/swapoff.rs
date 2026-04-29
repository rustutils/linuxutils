use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::swapoff::run(linuxutils_system::swapoff::Args::parse())
}
