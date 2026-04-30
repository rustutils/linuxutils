use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use linuxutils_blkid::{Cache, ResolveError, Tag};
use std::process::ExitCode;

/// Resolve a `LABEL=` / `UUID=` / `PARTLABEL=` / `PARTUUID=` /
/// `ID=` tag to the device path it refers to.
///
/// Lookups go through the udev `/dev/disk/by-*` symlink layer.
/// Filesystem-signature probing is not implemented; if the host's
/// udev didn't populate the symlinks, no resolution happens.
///
/// Exit status: 0 on success, 1 on not-found, 2 on usage error.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "findfs", about = "Find a filesystem by label or UUID")]
pub struct Args {
    /// Tag to resolve, e.g. `LABEL=root` or `UUID=12345...`. Anything
    /// not in `KIND=value` form is echoed verbatim, matching upstream
    /// findfs(8).
    #[arg(value_name = "NAME=value")]
    spec: String,
}

pub fn run(args: Args) -> ExitCode {
    // If the input isn't a tag, print it verbatim — matches upstream:
    // "If the input is not in the format of NAME=value, then the input
    //  will be copied to the output without any modification."
    let Some(tag) = Tag::parse(&args.spec) else {
        println!("{}", args.spec);
        return ExitCode::SUCCESS;
    };

    let cache = Cache::new();
    match cache.resolve(&tag) {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(ResolveError::NotFound) => {
            eprintln!("findfs: unable to resolve '{}'", args.spec);
            ExitCode::FAILURE
        }
        Err(ResolveError::NotATag) => {
            // Unreachable — Tag::parse already returned Some.
            eprintln!("findfs: not a recognised tag: {}", args.spec);
            ExitCode::from(2)
        }
        Err(ResolveError::Io(e)) => {
            eprintln!("findfs: i/o error resolving '{}': {e}", args.spec);
            ExitCode::FAILURE
        }
    }
}
