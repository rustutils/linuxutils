use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_term::mesg::run(linuxutils_term::mesg::Args::parse())
}
