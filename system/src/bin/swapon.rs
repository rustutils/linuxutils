use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::swapon::run(linuxutils_system::swapon::Args::parse())
}
