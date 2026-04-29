use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::pivot_root::run(linuxutils_system::pivot_root::Args::parse())
}
