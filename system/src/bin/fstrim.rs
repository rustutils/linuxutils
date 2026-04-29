use clap::Parser;
use linuxutils_system::fstrim::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
