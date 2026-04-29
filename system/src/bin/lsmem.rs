use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::lsmem::run(linuxutils_system::lsmem::Args::parse())
}
