use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::setpgid::run(linuxutils_system::setpgid::Args::parse())
}
