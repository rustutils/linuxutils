use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_renice::run(linuxutils_renice::Args::parse())
}
