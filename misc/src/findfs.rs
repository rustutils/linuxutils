use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty().section(8);

use clap::Parser;
use linuxutils_blkid::{Cache, ResolveError, Tag};
use std::process::ExitCode;

/// Find the device that a LABEL=, UUID=, PARTLABEL=, PARTUUID= or ID=
/// tag refers to.
///
/// Lookups use the udev symlinks in /dev/disk/by-*. Probing for
/// filesystem signatures is not implemented, so if udev hasn't created
/// the symlinks, nothing is found.
///
/// Exit status: 0 on success, 1 on not-found, 2 on usage error.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "findfs", about = "Find a filesystem by label or UUID")]
pub struct Args {
    /// Tag to look up, such as LABEL=root or UUID=12345... Anything not
    /// in KIND=value form is printed back unchanged, as upstream findfs(8)
    /// does.
    #[arg(value_name = "NAME=value")]
    spec: String,
}

pub fn run(args: Args) -> ExitCode {
    // If the input isn't a tag, print it verbatim. This matches upstream:
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
            // Unreachable: Tag::parse already returned Some.
            eprintln!("findfs: not a recognised tag: {}", args.spec);
            ExitCode::from(2)
        }
        Err(ResolveError::Io(e)) => {
            eprintln!("findfs: i/o error resolving '{}': {e}", args.spec);
            ExitCode::FAILURE
        }
    }
}
