use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::fadvise::run(linuxutils_system::fadvise::Args::parse())
}
