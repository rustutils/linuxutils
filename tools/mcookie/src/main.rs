use clap::Parser;
use std::{fs::File, io, path::PathBuf, process::ExitCode};

/// Generate magic cookies for xauth.
///
/// Generates a 128-bit random hexadecimal number for use with the X authority
/// system.
#[derive(Parser)]
#[command(name = "mcookie", version, about)]
struct Args {
    /// Use this file as an additional source of randomness.
    /// When file is '-', characters are read from standard input.
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Read from file only this number of bytes.
    #[arg(short, long)]
    max_size: Option<u64>,

    /// Inform where randomness originated.
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let result = if let Some(ref path) = args.file {
        if path.as_os_str() == "-" {
            let mut stdin = io::stdin().lock();
            rustutils_mcookie::mcookie(
                Some(&mut stdin),
                args.max_size,
                args.verbose,
            )
        } else {
            match File::open(path) {
                Ok(mut f) => rustutils_mcookie::mcookie(
                    Some(&mut f),
                    args.max_size,
                    args.verbose,
                ),
                Err(e) => {
                    eprintln!("mcookie: {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            }
        }
    } else {
        rustutils_mcookie::mcookie(None, args.max_size, args.verbose)
    };

    match result {
        Ok(cookie) => {
            println!("{cookie}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mcookie: {e}");
            ExitCode::FAILURE
        }
    }
}
