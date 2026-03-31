use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_isosize::run(linuxutils_isosize::Args::parse())
}
