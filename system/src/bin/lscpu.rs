use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::lscpu::run(linuxutils_system::lscpu::Args::parse())
}
