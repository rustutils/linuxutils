use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::rename::run(linuxutils_misc::rename::Args::parse())
}
