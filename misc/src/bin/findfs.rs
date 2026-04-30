use clap::Parser;
use linuxutils_misc::findfs::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
