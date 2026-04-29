use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::logger::run(linuxutils_misc::logger::Args::parse())
}
