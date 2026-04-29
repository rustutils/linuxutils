use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_text::column::run(linuxutils_text::column::Args::parse())
}
