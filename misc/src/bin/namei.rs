use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_misc::namei::run(linuxutils_misc::namei::Args::parse())
}
