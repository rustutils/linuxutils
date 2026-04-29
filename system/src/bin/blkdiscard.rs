use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::blkdiscard::run(linuxutils_system::blkdiscard::Args::parse())
}
