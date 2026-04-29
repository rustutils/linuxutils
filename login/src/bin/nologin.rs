use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_login::nologin::run(linuxutils_login::nologin::Args::parse())
}
