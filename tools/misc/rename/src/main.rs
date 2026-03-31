use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_rename::run(linuxutils_rename::Args::parse())
}
