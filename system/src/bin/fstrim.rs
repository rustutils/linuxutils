use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::fstrim::run(linuxutils_system::fstrim::Args::parse())
}
