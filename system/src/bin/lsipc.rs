use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::lsipc::run(linuxutils_system::lsipc::Args::parse())
}
