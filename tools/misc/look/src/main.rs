use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_look::run(linuxutils_look::Args::parse())
}
