use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::rfkill::run(linuxutils_system::rfkill::Args::parse())
}
