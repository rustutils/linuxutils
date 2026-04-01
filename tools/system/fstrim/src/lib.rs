use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead},
    os::unix::io::AsRawFd,
    process::ExitCode,
};

const FITRIM: libc::c_ulong = 0xc0185879;

#[repr(C)]
struct FstrimRange {
    start: u64,
    len: u64,
    minlen: u64,
}

#[derive(Parser)]
#[command(
    name = "fstrim",
    about = "Discard unused blocks on a mounted filesystem"
)]
pub struct Args {
    /// Trim all mounted filesystems that support discard
    #[arg(short = 'a', long)]
    all: bool,

    /// Trim all mounted filesystems mentioned in /etc/fstab
    #[arg(short = 'A', long)]
    fstab: bool,

    /// Byte offset to start trimming from
    #[arg(short = 'o', long, default_value = "0")]
    offset: u64,

    /// Number of bytes to trim
    #[arg(short = 'l', long)]
    length: Option<u64>,

    /// Minimum contiguous free range to discard
    #[arg(short = 'm', long, default_value = "0")]
    minimum: u64,

    /// Comma-separated filesystem type filter
    #[arg(short = 't', long)]
    types: Option<String>,

    /// Print number of discarded bytes
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Do everything except the actual FITRIM ioctl
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Mountpoint to trim (not used with --all or --fstab)
    mountpoint: Option<String>,
}

fn do_fstrim(
    path: &str,
    offset: u64,
    length: u64,
    minlen: u64,
    dry_run: bool,
) -> io::Result<u64> {
    let f = File::open(path)?;
    let mut range = FstrimRange {
        start: offset,
        len: length,
        minlen,
    };

    if dry_run {
        return Ok(0);
    }

    let ret = unsafe { libc::ioctl(f.as_raw_fd(), FITRIM, &mut range) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(range.len)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    const TIB: u64 = 1024 * GIB;

    if bytes >= TIB {
        format!("{:.1} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

struct MountEntry {
    mountpoint: String,
    fstype: String,
}

fn read_mountinfo() -> io::Result<Vec<MountEntry>> {
    let file = File::open("/proc/self/mountinfo")?;
    let mut entries = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        // Format: id parent major:minor root mountpoint options ... - fstype source super_options
        if let Some(sep_idx) = fields.iter().position(|&f| f == "-")
            && sep_idx + 1 < fields.len()
            && fields.len() > 4
        {
            let mountpoint = fields[4].to_string();
            let fstype = fields[sep_idx + 1].to_string();
            entries.push(MountEntry { mountpoint, fstype });
        }
    }

    Ok(entries)
}

fn read_fstab_mountpoints() -> io::Result<Vec<String>> {
    let file = File::open("/etc/fstab")?;
    let mut mountpoints = Vec::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 4 {
            let mountpoint = fields[1];
            let options = fields[3];
            if options.contains("X-fstrim.notrim") {
                continue;
            }
            if mountpoint != "none" && mountpoint != "swap" {
                mountpoints.push(mountpoint.to_string());
            }
        }
    }

    Ok(mountpoints)
}

fn type_matches(fstype: &str, filter: &Option<String>) -> bool {
    let Some(filter) = filter else {
        return fstype != "autofs";
    };

    for entry in filter.split(',') {
        if let Some(excluded) = entry.strip_prefix("no") {
            if fstype == excluded {
                return false;
            }
        } else if fstype != entry {
            return false;
        }
    }
    true
}

pub fn run(args: Args) -> ExitCode {
    let length = args.length.unwrap_or(u64::MAX);

    if args.all || args.fstab {
        let mounts = match read_mountinfo() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("fstrim: failed to read mountinfo: {e}");
                return ExitCode::FAILURE;
            }
        };

        let targets: Vec<&MountEntry> = if args.fstab {
            let fstab_mounts = read_fstab_mountpoints().unwrap_or_default();
            mounts
                .iter()
                .filter(|m| {
                    fstab_mounts.contains(&m.mountpoint)
                        && type_matches(&m.fstype, &args.types)
                })
                .collect()
        } else {
            mounts
                .iter()
                .filter(|m| type_matches(&m.fstype, &args.types))
                .collect()
        };

        let mut successes = 0;
        let mut failures = 0;

        for mount in &targets {
            match do_fstrim(
                &mount.mountpoint,
                args.offset,
                length,
                args.minimum,
                args.dry_run,
            ) {
                Ok(trimmed) => {
                    successes += 1;
                    if args.verbose {
                        println!(
                            "{}: {} ({trimmed} bytes) trimmed",
                            mount.mountpoint,
                            format_bytes(trimmed)
                        );
                    }
                }
                Err(e) => {
                    let errno = e.raw_os_error().unwrap_or(0);
                    // Silently skip unsupported/read-only (EOPNOTSUPP, EROFS, EACCES)
                    if errno == libc::EOPNOTSUPP
                        || errno == libc::EROFS
                        || errno == libc::EACCES
                    {
                        continue;
                    }
                    eprintln!("fstrim: {}: {e}", mount.mountpoint);
                    failures += 1;
                }
            }
        }

        if failures > 0 && successes == 0 {
            ExitCode::from(32)
        } else if failures > 0 {
            ExitCode::from(64)
        } else {
            ExitCode::SUCCESS
        }
    } else {
        let mountpoint = match &args.mountpoint {
            Some(m) => m.as_str(),
            None => {
                eprintln!("fstrim: no mountpoint specified");
                return ExitCode::FAILURE;
            }
        };

        match do_fstrim(
            mountpoint,
            args.offset,
            length,
            args.minimum,
            args.dry_run,
        ) {
            Ok(trimmed) => {
                if args.verbose {
                    println!(
                        "{mountpoint}: {} ({trimmed} bytes) trimmed",
                        format_bytes(trimmed)
                    );
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("fstrim: {mountpoint}: {e}");
                ExitCode::FAILURE
            }
        }
    }
}
