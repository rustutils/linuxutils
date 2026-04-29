use clap::Parser;
use linuxutils_system::dmesg::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
