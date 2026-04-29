use clap::Parser;
use linuxutils_system::ipcmk::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
