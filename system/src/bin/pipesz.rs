use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::pipesz::run(linuxutils_system::pipesz::Args::parse())
}
