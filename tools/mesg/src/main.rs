use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    rustutils_mesg::run(rustutils_mesg::Args::parse())
}
