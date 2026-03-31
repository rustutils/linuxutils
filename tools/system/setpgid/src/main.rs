use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_setpgid::run(linuxutils_setpgid::Args::parse())
}
