use clap::Parser;
use std::{
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
    process::ExitCode,
};

/// Reverse lines characterwise.
#[derive(Parser)]
#[command(name = "rev", version, about)]
struct Args {
    /// Use '\0' as the line separator instead of newline.
    #[arg(short = '0', long)]
    zero: bool,

    /// Files to read. If none are given, reads from standard input.
    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    if args.files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();
        if let Err(e) = rustutils_rev::rev(reader, &mut writer, args.zero) {
            eprintln!("rev: {e}");
            return ExitCode::FAILURE;
        }
    } else {
        for path in &args.files {
            let file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("rev: {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            };
            let reader = BufReader::new(file);
            if let Err(e) = rustutils_rev::rev(reader, &mut writer, args.zero) {
                eprintln!("rev: {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
