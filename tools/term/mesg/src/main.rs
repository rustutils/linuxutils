use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_mesg::run(linuxutils_mesg::Args::parse())
}
