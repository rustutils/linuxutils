use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::fs::{CWD, RenameFlags, renameat_with};
use std::process::ExitCode;

/// Atomically exchange two filesystem paths.
///
/// Wraps `renameat2(2)` with the `RENAME_EXCHANGE` flag. Both paths
/// must exist and live on the same filesystem.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "exch", about = "Atomically exchange two filesystem paths")]
pub struct Args {
    /// First path
    path1: String,

    /// Second path
    path2: String,
}

pub fn run(args: Args) -> ExitCode {
    match renameat_with(
        CWD,
        args.path1.as_str(),
        CWD,
        args.path2.as_str(),
        RenameFlags::EXCHANGE,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "exch: {} <-> {}: {}",
                args.path1,
                args.path2,
                std::io::Error::from(e)
            );
            ExitCode::FAILURE
        }
    }
}
