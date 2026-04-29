use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_text::colrm::run(linuxutils_text::colrm::Args::parse())
}
