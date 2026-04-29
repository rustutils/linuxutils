use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    linuxutils_text::hexdump::run(linuxutils_text::hexdump::Args::parse())
}
