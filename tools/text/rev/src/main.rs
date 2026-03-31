use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_rev::run(linuxutils_rev::Args::parse())
}
