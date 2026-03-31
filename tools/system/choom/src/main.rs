use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_choom::run(linuxutils_choom::Args::parse())
}
