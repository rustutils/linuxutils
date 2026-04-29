use clap::Parser;
use linuxutils_system::swapon::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
