use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_fallocate::run(linuxutils_fallocate::Args::parse())
}
