use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::copyfilerange::run(linuxutils_misc::copyfilerange::Args::parse())
}
