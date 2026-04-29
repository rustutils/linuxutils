use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::fallocate::run(linuxutils_system::fallocate::Args::parse())
}
