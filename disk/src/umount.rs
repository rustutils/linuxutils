use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::{ArgAction, Parser};
use linuxutils_blkid::Cache as BlkidCache;
use linuxutils_mount::{
    MountTable, Spec, Tag, UnmountFlags, syscall::umount as do_umount,
};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::ExitCode,
};

/// Unmount a filesystem.
///
/// Each positional argument can be a mount point, a source device,
/// or a `LABEL=`/`UUID=` tag. Tags are resolved via libblkid; sources
/// are looked up in `/proc/self/mountinfo` to find the target mount
/// point that's actually unmounted.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "umount", about = "Unmount filesystems")]
pub struct Args {
    /// Force the unmount even if the device is busy (`MNT_FORCE`)
    #[arg(short = 'f', long, action = ArgAction::SetTrue)]
    force: bool,

    /// Lazy unmount — detach now, free when no longer busy
    /// (`MNT_DETACH`)
    #[arg(short = 'l', long, action = ArgAction::SetTrue)]
    lazy: bool,

    /// On failure, try a `mount -o remount,ro` fallback so the
    /// data is still flushed and the fs is read-only afterwards
    #[arg(short = 'r', long = "read-only", action = ArgAction::SetTrue)]
    read_only_fallback: bool,

    /// Do not write to /etc/mtab on success (no-op for us — we
    /// don't write mtab — but accepted for command-line
    /// compatibility)
    #[arg(short = 'n', long = "no-mtab", action = ArgAction::SetTrue)]
    no_mtab: bool,

    /// Print what we're doing
    #[arg(short = 'v', long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Targets to unmount: mount point, device path, or tag
    /// (`LABEL=foo`, `UUID=...`)
    #[arg(required = true, value_name = "TARGET")]
    targets: Vec<String>,
}

fn unmount_flags(args: &Args) -> UnmountFlags {
    let mut f = UnmountFlags::empty();
    if args.force {
        f |= UnmountFlags::FORCE;
    }
    if args.lazy {
        f |= UnmountFlags::DETACH;
    }
    f
}

/// Resolve a positional argument to the mount-point path we should
/// pass to `umount2(2)`.
///
/// The kernel's `umount2` only accepts mount points (not source
/// devices), so we need a mountinfo lookup whenever the user
/// specifies a source rather than a target.
fn resolve_target(arg: &str) -> Result<PathBuf, String> {
    // 1) Tag form (LABEL=/UUID=/...): resolve to a device path via
    //    blkid, then fall through to source-lookup logic.
    let device_form: PathBuf = if let Some(tag) = Tag::parse(arg) {
        BlkidCache::new()
            .resolve(&tag)
            .map_err(|e| format!("can't resolve {arg}: {e}"))?
    } else {
        PathBuf::from(arg)
    };

    // 2) If the user-supplied path *is* a current mount point in
    //    /proc/self/mountinfo, use it directly.
    let mountinfo = read_host_mountinfo()?;
    let canonical = std::fs::canonicalize(&device_form).unwrap_or(device_form);
    if mountinfo.find_target(&canonical).is_some() {
        return Ok(canonical);
    }

    // 3) Otherwise, the canonical form must be a source — look it
    //    up by source. There can be multiple instances; we pick the
    //    first (matches upstream's default behaviour without
    //    `--all-targets`).
    if let Some(entry) = mountinfo
        .iter()
        .find(|e| matches!(&e.source, Spec::Path(p) if p == &canonical))
    {
        return Ok(entry.target.clone());
    }

    Err(format!("{arg}: not a mount point or known mount source"))
}

fn read_host_mountinfo() -> Result<MountTable, String> {
    let f = File::open("/proc/self/mountinfo")
        .map_err(|e| format!("/proc/self/mountinfo: {e}"))?;
    MountTable::parse_mountinfo(BufReader::new(f))
        .map_err(|e| format!("/proc/self/mountinfo: {e}"))
}

fn try_remount_ro(target: &Path) -> std::io::Result<()> {
    use linuxutils_mount::{MountFlags, syscall::remount};
    remount(target, MountFlags::RDONLY, None)
}

pub fn run(args: Args) -> ExitCode {
    let flags = unmount_flags(&args);
    let mut had_error = false;

    for spec in &args.targets {
        let target = match resolve_target(spec) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("umount: {e}");
                had_error = true;
                continue;
            }
        };

        if args.verbose {
            eprintln!("umount: unmounting {}", target.display());
        }

        match do_umount(&target, flags) {
            Ok(()) => {}
            Err(e) if args.read_only_fallback => {
                eprintln!(
                    "umount: {}: {e}, trying remount,ro",
                    target.display()
                );
                if let Err(re) = try_remount_ro(&target) {
                    eprintln!(
                        "umount: {}: remount,ro failed: {re}",
                        target.display()
                    );
                    had_error = true;
                }
            }
            Err(e) => {
                eprintln!("umount: {}: {e}", target.display());
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
