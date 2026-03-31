use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_setsid::run(linuxutils_setsid::Args::parse())
}
