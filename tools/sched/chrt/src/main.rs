use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_chrt::run(linuxutils_chrt::Args::parse())
}
