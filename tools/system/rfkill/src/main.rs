use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_rfkill::run(linuxutils_rfkill::Args::parse())
}
