use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::lslocks::run(linuxutils_misc::lslocks::Args::parse())
}
