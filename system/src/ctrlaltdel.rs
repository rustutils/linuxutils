use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty().section(8);

use clap::Parser;
use std::process::ExitCode;

const PROC_PATH: &str = "/proc/sys/kernel/ctrl-alt-del";

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "ctrlaltdel",
    about = "Set the function of the Ctrl-Alt-Del combination"
)]
pub struct Args {
    /// Mode: hard or soft (omit to query current setting)
    mode: Option<String>,
}

pub fn run(args: Args) -> ExitCode {
    match args.mode.as_deref() {
        None => match std::fs::read_to_string(PROC_PATH) {
            Ok(val) => {
                match val.trim() {
                    "0" => println!("soft"),
                    "1" => println!("hard"),
                    other => println!("{other}"),
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("ctrlaltdel: failed to read {PROC_PATH}: {e}");
                ExitCode::FAILURE
            }
        },
        Some("hard") => {
            if let Err(e) = std::fs::write(PROC_PATH, "1\n") {
                eprintln!("ctrlaltdel: failed to set hard: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some("soft") => {
            if let Err(e) = std::fs::write(PROC_PATH, "0\n") {
                eprintln!("ctrlaltdel: failed to set soft: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!(
                "ctrlaltdel: unknown mode '{other}' (expected 'hard' or 'soft')"
            );
            ExitCode::FAILURE
        }
    }
}
