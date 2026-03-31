use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_colrm::run(linuxutils_colrm::Args::parse())
}
