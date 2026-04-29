use clap::Parser;
use linuxutils_system::ipcs::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
