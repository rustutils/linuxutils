use clap::Parser;
use linuxutils_system::losetup::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
