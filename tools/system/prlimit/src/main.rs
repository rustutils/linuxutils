use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_prlimit::run(linuxutils_prlimit::Args::parse())
}
