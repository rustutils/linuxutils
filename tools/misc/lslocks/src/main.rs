use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_lslocks::run(linuxutils_lslocks::Args::parse())
}
