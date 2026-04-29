use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_text::line::run(linuxutils_text::line::Args::parse())
}
