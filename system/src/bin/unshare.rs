use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::unshare::run(linuxutils_system::unshare::Args::parse())
}
