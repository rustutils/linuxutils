use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::kill::run(linuxutils_misc::kill::Args::parse())
}
