use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_disk::isosize::run(linuxutils_disk::isosize::Args::parse())
}
