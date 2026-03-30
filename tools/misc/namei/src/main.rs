use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_namei::run(rustutils_namei::Args::parse())
}
