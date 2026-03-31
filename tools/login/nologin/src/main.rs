use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_nologin::run(linuxutils_nologin::Args::parse())
}
