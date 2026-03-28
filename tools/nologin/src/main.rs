use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_nologin::run(rustutils_nologin::Args::parse())
}
