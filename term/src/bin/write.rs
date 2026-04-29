use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_term::write::run(linuxutils_term::write::Args::parse())
}
