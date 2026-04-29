use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::ipcrm::run(linuxutils_system::ipcrm::Args::parse())
}
