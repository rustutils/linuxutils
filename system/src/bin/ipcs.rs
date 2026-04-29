use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::ipcs::run(linuxutils_system::ipcs::Args::parse())
}
