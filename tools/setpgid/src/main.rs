use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_setpgid::run(rustutils_setpgid::Args::parse())
}
