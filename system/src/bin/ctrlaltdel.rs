use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::ctrlaltdel::run(linuxutils_system::ctrlaltdel::Args::parse())
}
