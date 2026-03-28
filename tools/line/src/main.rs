use clap::Parser;
use std::{io, process::ExitCode};

/// Read one line from standard input and write it to standard output.
#[derive(Parser)]
#[command(name = "line", version, about)]
struct Args {}

fn main() -> ExitCode {
    let _args = Args::parse();
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut stdout = io::stdout().lock();

    match rustutils_line::line(reader, &mut stdout) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("line: {e}");
            ExitCode::FAILURE
        }
    }
}
