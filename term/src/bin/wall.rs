use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_term::wall::run(linuxutils_term::wall::Args::parse())
}
