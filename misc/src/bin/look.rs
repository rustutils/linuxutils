use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::look::run(linuxutils_misc::look::Args::parse())
}
