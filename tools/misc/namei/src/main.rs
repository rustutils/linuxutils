use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_namei::run(linuxutils_namei::Args::parse())
}
