use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::dmesg::run(linuxutils_system::dmesg::Args::parse())
}
