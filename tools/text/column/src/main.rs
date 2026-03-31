use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_column::run(linuxutils_column::Args::parse())
}
