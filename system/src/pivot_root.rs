use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty().section(8);

use clap::Parser;
use std::process::ExitCode;

/// Change the root filesystem via pivot_root(2).
///
/// Moves the current root to PUT_OLD and makes NEW_ROOT the new root.
/// Both paths must be directories. NEW_ROOT must be a mount point, and
/// PUT_OLD must be at or under NEW_ROOT.
#[derive(Parser, Debug)]
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
