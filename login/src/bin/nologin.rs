use clap::Parser;
use linuxutils_login::nologin::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
