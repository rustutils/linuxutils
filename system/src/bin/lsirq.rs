use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::lsirq::run(linuxutils_system::lsirq::Args::parse())
}
