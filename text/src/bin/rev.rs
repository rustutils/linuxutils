use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_text::rev::run(linuxutils_text::rev::Args::parse())
}
