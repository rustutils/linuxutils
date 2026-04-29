use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::cal::run(linuxutils_misc::cal::Args::parse())
}
