use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::waitpid::run(linuxutils_misc::waitpid::Args::parse())
}
