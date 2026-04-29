use clap::Parser;
use rustix::{
    fs::{Mode, fchmod, fstat},
    termios::isatty,
};
use std::{io, process::ExitCode};

#[derive(Parser)]
#[command(
    name = "mesg",
    version,
    about = "Display (or do not display) messages from other users"
)]
pub struct Args {
    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// y to allow messages, n to disallow
    #[arg(value_parser = parse_yn)]
    action: Option<bool>,
}

fn parse_yn(s: &str) -> Result<bool, String> {
    match s {
        "y" => Ok(true),
        "n" => Ok(false),
        _ => Err(format!("invalid argument: '{s}' (expected 'y' or 'n')")),
    }
}

pub fn run(args: Args) -> ExitCode {
    let stderr = io::stderr();
    if !isatty(&stderr) {
        if args.verbose {
            eprintln!("mesg: not a terminal");
        }
        return ExitCode::from(2);
    }

    let stat = match fstat(&stderr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("mesg: {e}");
            return ExitCode::from(2);
        }
    };

    let current_mode = Mode::from_raw_mode(stat.st_mode);
    let group_write = current_mode.contains(Mode::WGRP);

    match args.action {
        None => {
            // Display current status.
            if group_write {
                eprintln!("is y");
                ExitCode::SUCCESS
            } else {
                eprintln!("is n");
                ExitCode::from(1)
            }
        }
        Some(allow) => {
            let new_mode = if allow {
                current_mode | Mode::WGRP
            } else {
                current_mode & !Mode::WGRP
            };

            if let Err(e) = fchmod(&stderr, new_mode) {
                eprintln!("mesg: {e}");
                return ExitCode::from(2);
            }

            if args.verbose {
                if allow {
                    eprintln!("mesg: write access to your terminal is allowed");
                } else {
                    eprintln!("mesg: write access to your terminal is denied");
                }
            }

            if allow {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
    }
}
