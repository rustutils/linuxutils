use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::setsid::run(linuxutils_system::setsid::Args::parse())
}
