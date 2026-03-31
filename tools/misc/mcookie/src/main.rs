use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_mcookie::run(linuxutils_mcookie::Args::parse())
}
