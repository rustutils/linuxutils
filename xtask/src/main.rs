use clap::Parser;
use std::process::ExitCode;

mod mangen;

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Developer-only tasks for the linuxutils workspace"
)]
enum Cmd {
    /// Generate man pages for every linuxutils tool.
    Mangen {
        /// Directory to write the man pages into, in man1 and man8
        /// subdirectories.
        #[arg(default_value = "target/man")]
        output: String,
    },
}

fn main() -> ExitCode {
    match Cmd::parse() {
        Cmd::Mangen { output } => mangen::run(&output),
    }
}
