use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::process::ExitCode;

/// Change the root filesystem via pivot_root(2).
///
/// Moves the current root to `put_old` and makes `new_root` the new root.
/// Both paths must be directories; `new_root` must be a mount point and
/// `put_old` must be at or under `new_root`.
#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "pivot_root", about = "Change the root filesystem")]
pub struct Args {
    /// New root filesystem path
    new_root: String,

    /// Where to move the old root
    put_old: String,
}

pub fn run(args: Args) -> ExitCode {
    if let Err(e) = rustix::process::pivot_root(&args.new_root, &args.put_old) {
        eprintln!(
            "pivot_root: failed to pivot from '{}' to '{}': {}",
            args.new_root,
            args.put_old,
            std::io::Error::from(e)
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
