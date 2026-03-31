use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_line::run(linuxutils_line::Args::parse())
}
