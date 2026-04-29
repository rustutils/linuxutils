use clap::Parser;
use std::process::ExitCode;

mod mangen;

#[derive(Parser)]
#[command(name = "xtask", about = "Developer-only tasks for the linuxutils workspace")]
enum Cmd {
    /// Generate man pages for every linuxutils tool.
    Mangen {
        /// Directory to write `<tool>.1` files into.
        #[arg(default_value = "target/man")]
        output: String,
    },
}

fn main() -> ExitCode {
    match Cmd::parse() {
        Cmd::Mangen { output } => mangen::run(&output),
    }
}
