use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_system::ipcmk::run(linuxutils_system::ipcmk::Args::parse())
}
