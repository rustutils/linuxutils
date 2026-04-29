use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::choom::run(linuxutils_system::choom::Args::parse())
}
