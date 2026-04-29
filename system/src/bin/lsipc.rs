use clap::Parser;
use linuxutils_system::lsipc::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
