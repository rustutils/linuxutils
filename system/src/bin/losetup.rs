use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::losetup::run(linuxutils_system::losetup::Args::parse())
}
