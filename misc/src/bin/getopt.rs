use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::getopt::run(linuxutils_misc::getopt::Args::parse())
}
