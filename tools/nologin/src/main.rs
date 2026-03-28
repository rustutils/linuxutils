use clap::Parser;
use std::{io, path::PathBuf, process::ExitCode};

/// Politely refuse a login.
///
/// Displays a message that an account is not available and exits non-zero.
/// Intended as a replacement shell field to deny login access to an account.
#[derive(Parser)]
#[command(name = "nologin", version, about)]
struct Args {
    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    command: Option<String>,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    init_file: Option<PathBuf>,

    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    interactive: bool,

    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    login: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    noprofile: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    norc: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    posix: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    rcfile: Option<PathBuf>,

    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    restricted: bool,
}

fn main() -> ExitCode {
    let _args = Args::parse();
    let mut stderr = io::stderr().lock();

    if let Err(e) = rustutils_nologin::nologin(&mut stderr) {
        eprintln!("nologin: {e}");
    }

    ExitCode::from(1)
}
