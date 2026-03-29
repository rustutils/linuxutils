use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_choom::run(rustutils_choom::Args::parse())
}
