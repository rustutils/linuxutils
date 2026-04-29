use clap::Parser;
use linuxutils_term::mesg::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    run(Args::parse())
}
