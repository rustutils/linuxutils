use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::mcookie::run(linuxutils_misc::mcookie::Args::parse())
}
