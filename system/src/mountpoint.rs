use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::fs::{FileType, lstat, major, minor, stat};
use std::{
    io::{self, BufRead},
    path::{Path, PathBuf},
    process::ExitCode,
};

const EXIT_USAGE: u8 = 1;

const EXIT_NOTMOUNT: u8 = 32;

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "mountpoint",
    version,
    about = "See if a directory or file is a mountpoint"
)]
pub struct Args {
    /// Show the major and minor numbers of the device mounted on the directory
    #[arg(short = 'd', long = "fs-devno", conflicts_with = "devno")]
    fs_devno: bool,

    /// Be quiet - don't print anything
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Do not follow symbolic link if it is the last element of the path
    #[arg(long = "nofollow")]
    nofollow: bool,

    /// Show the major and minor numbers of the given block device
    #[arg(short = 'x', long = "devno", conflicts_with_all = ["fs_devno", "quiet"])]
    devno: bool,

    /// Directory, file, or device to check
    pub path: PathBuf,
}

pub fn run(args: Args) -> ExitCode {
    // Upstream rejects --devno with --nofollow before doing any
    // I/O — the two options apply to opposite halves of the
    // resolution chain (one needs a block device, the other forbids
    // following symlinks to one).
    if args.devno && args.nofollow {
        eprintln!("mountpoint: --devno and --nofollow are mutually exclusive");
        return ExitCode::from(EXIT_USAGE);
    }

    if args.devno {
        return show_devno(&args);
    }

    let st = match do_stat(&args.path, args.nofollow) {
        Ok(s) => s,
        Err(e) => {
            if !args.quiet {
                eprintln!("mountpoint: {}: {e}", args.path.display());
            }
            return ExitCode::from(1);
        }
    };

    if args.fs_devno {
        if let Some((maj, min)) = mountinfo_devno(&args.path, args.nofollow) {
            println!("{maj}:{min}");
        } else {
            println!("{}:{}", major(st.st_dev), minor(st.st_dev));
        }
        return ExitCode::SUCCESS;
    }

    let is_mount = is_mountpoint(&args.path, args.nofollow, &st);

    if !args.quiet {
        if is_mount {
            println!("{} is a mountpoint", args.path.display());
        } else {
            println!("{} is not a mountpoint", args.path.display());
        }
    }

    if is_mount {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(EXIT_NOTMOUNT)
    }
}

fn do_stat(
    path: &PathBuf,
    nofollow: bool,
) -> Result<rustix::fs::Stat, rustix::io::Errno> {
    if nofollow { lstat(path) } else { stat(path) }
}

fn show_devno(args: &Args) -> ExitCode {
    let st = match do_stat(&args.path, false) {
        Ok(s) => s,
        Err(e) => {
            if !args.quiet {
                eprintln!("mountpoint: {}: {e}", args.path.display());
            }
            return ExitCode::from(1);
        }
    };

    // Check if it's a block device.
    if !FileType::from_raw_mode(st.st_mode).is_block_device() {
        if !args.quiet {
            eprintln!(
                "mountpoint: {}: not a block device",
                args.path.display()
            );
        }
        return ExitCode::from(EXIT_NOTMOUNT);
    }

    println!("{}:{}", major(st.st_rdev), minor(st.st_rdev));
    ExitCode::SUCCESS
}

/// Check if a path is a mountpoint. Under `--nofollow`, a final
/// component that is itself a symlink is *never* a mountpoint —
/// mounts always live on the resolved target. Otherwise look the
/// path up in `/proc/self/mountinfo` and fall back to the
/// classic "different st_dev from parent" trick if the path
/// hasn't been canonicalised.
fn is_mountpoint(path: &Path, nofollow: bool, st: &rustix::fs::Stat) -> bool {
    if nofollow && FileType::from_raw_mode(st.st_mode).is_symlink() {
        return false;
    }
    if find_in_mountinfo(path, nofollow).is_some() {
        return true;
    }
    match stat(path) {
        Ok(target_st) => is_mountpoint_by_stat(&target_st, path),
        Err(_) => false,
    }
}

/// Look up a path in `/proc/self/mountinfo` and return its
/// major:minor if found. Under `--nofollow`, the path is matched
/// without canonicalisation so a symlink isn't silently resolved
/// to its mount target.
fn find_in_mountinfo(path: &Path, nofollow: bool) -> Option<(u32, u32)> {
    let resolved = if nofollow {
        path.to_path_buf()
    } else {
        std::fs::canonicalize(path).ok()?
    };
    let resolved = resolved.to_string_lossy();

    let file = std::fs::File::open("/proc/self/mountinfo").ok()?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let Ok(line) = line else { continue };
        // mountinfo format: ID PARENT_ID MAJOR:MINOR ROOT MOUNT_POINT ...
        let mut fields = line.split_whitespace();
        let Some(_id) = fields.next() else { continue };
        let Some(_parent) = fields.next() else {
            continue;
        };
        let Some(devno) = fields.next() else { continue };
        let Some(_root) = fields.next() else { continue };
        let Some(mount_point) = fields.next() else {
            continue;
        };

        let decoded = unescape_mountinfo(mount_point);
        if *resolved == decoded {
            let (maj_s, min_s) = devno.split_once(':')?;
            let maj = maj_s.parse().ok()?;
            let min = min_s.parse().ok()?;
            return Some((maj, min));
        }
    }

    None
}

/// Look up the device number from mountinfo for a given path.
fn mountinfo_devno(path: &Path, nofollow: bool) -> Option<(u32, u32)> {
    find_in_mountinfo(path, nofollow)
}

/// Fallback: a path is a mountpoint if its device ID differs from its parent's.
fn is_mountpoint_by_stat(st: &rustix::fs::Stat, path: &Path) -> bool {
    let parent = path.join("..");
    match stat(&parent) {
        Ok(parent_st) => st.st_dev != parent_st.st_dev,
        Err(_) => false,
    }
}

/// Unescape octal sequences like \040 in mountinfo paths.
fn unescape_mountinfo(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let oct: String = chars.by_ref().take(3).collect();
            if let Ok(byte) = u8::from_str_radix(&oct, 8) {
                result.push(byte as char);
            } else {
                result.push('\\');
                result.push_str(&oct);
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescape_simple() {
        assert_eq!(unescape_mountinfo("/mnt/my\\040drive"), "/mnt/my drive");
    }

    #[test]
    fn unescape_no_escapes() {
        assert_eq!(unescape_mountinfo("/mnt/data"), "/mnt/data");
    }

    #[test]
    fn root_is_mountpoint() {
        let st = stat("/").unwrap();
        assert!(is_mountpoint(Path::new("/"), false, &st));
    }
}
