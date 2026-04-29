use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use cols::{Cols, print_table};
use std::{
    ffi::CString,
    fs::File,
    io::{self, BufRead},
    process::ExitCode,
};

const SWAP_FLAG_PREFER: libc::c_int = 0x8000;
const SWAP_FLAG_DISCARD: libc::c_int = 0x10000;
const SWAP_FLAG_DISCARD_ONCE: libc::c_int = 0x20000;
const SWAP_FLAG_DISCARD_PAGES: libc::c_int = 0x40000;

/// Enable devices and files for paging and swapping.
///
/// Calls swapon(2) to activate swap areas. Can enable individual devices
/// or all swap entries from /etc/fstab.
#[derive(Parser)]
#[command(
    name = "swapon",
    about = "Enable devices and files for paging and swapping"
)]
pub struct Args {
    /// Enable all swap entries from /etc/fstab
    #[arg(short = 'a', long)]
    all: bool,

    /// Enable discard (once, pages, or both if no policy given)
    #[arg(short = 'd', long, num_args = 0..=1, default_missing_value = "")]
    discard: Option<String>,

    /// Silently skip non-existent devices
    #[arg(short = 'e', long)]
    ifexists: bool,

    /// Set swap priority (0-32767)
    #[arg(short = 'p', long)]
    priority: Option<i32>,

    /// Display /proc/swaps summary (deprecated)
    #[arg(short = 's', long)]
    summary: bool,

    /// Display formatted table of swap areas
    #[arg(long, num_args = 0..=1, default_missing_value = "")]
    show: Option<String>,

    /// Suppress table headers
    #[arg(long)]
    noheadings: bool,

    /// Show sizes in bytes instead of human-friendly units
    #[arg(long)]
    bytes: bool,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Devices or files to enable
    devices: Vec<String>,
}

#[derive(Cols)]
struct SwapEntry {
    #[column(header = "NAME")]
    name: String,

    #[column(header = "TYPE")]
    swap_type: String,

    #[column(right, header = "SIZE")]
    size: String,

    #[column(right, header = "USED")]
    used: String,

    #[column(right, header = "PRIO")]
    prio: i32,
}

fn format_size(kib: u64, bytes: bool) -> String {
    if bytes {
        return (kib * 1024).to_string();
    }
    let b = kib * 1024;
    if b >= 1024 * 1024 * 1024 {
        format!("{:.1}G", b as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024 * 1024 {
        format!("{:.1}M", b as f64 / (1024.0 * 1024.0))
    } else if b >= 1024 {
        format!("{:.1}K", b as f64 / 1024.0)
    } else {
        format!("{b}B")
    }
}

fn read_proc_swaps(bytes: bool) -> io::Result<Vec<SwapEntry>> {
    let file = File::open("/proc/swaps")?;
    let mut entries = Vec::new();

    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
    {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 5 {
            let size_kib: u64 = fields[2].parse().unwrap_or(0);
            let used_kib: u64 = fields[3].parse().unwrap_or(0);
            entries.push(SwapEntry {
                name: fields[0].to_string(),
                swap_type: fields[1].to_string(),
                size: format_size(size_kib, bytes),
                used: format_size(used_kib, bytes),
                prio: fields[4].parse().unwrap_or(0),
            });
        }
    }

    Ok(entries)
}

fn read_fstab_swap_devices() -> Vec<String> {
    let Ok(file) = File::open("/etc/fstab") else {
        return Vec::new();
    };
    let mut devices = Vec::new();

    for line in io::BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim().to_string();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 3 && fields[2] == "swap" {
            let options = if fields.len() >= 4 { fields[3] } else { "" };
            if !options.split(',').any(|o| o == "noauto") {
                devices.push(fields[0].to_string());
            }
        }
    }

    devices
}

fn is_swap_active(device: &str) -> bool {
    let Ok(file) = File::open("/proc/swaps") else {
        return false;
    };
    io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .any(|line| {
            line.split_whitespace()
                .next()
                .is_some_and(|name| name == device)
        })
}

fn do_swapon(path: &str, flags: libc::c_int) -> io::Result<()> {
    let cpath =
        CString::new(path).map_err(|e| io::Error::other(e.to_string()))?;
    if unsafe { libc::swapon(cpath.as_ptr(), flags) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn run(args: Args) -> ExitCode {
    // Display modes (no swap activation)
    if args.show.is_some() || args.summary {
        if args.summary {
            match std::fs::read_to_string("/proc/swaps") {
                Ok(content) => {
                    print!("{content}");
                    return ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!("swapon: /proc/swaps: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }

        let entries = match read_proc_swaps(args.bytes) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("swapon: /proc/swaps: {e}");
                return ExitCode::FAILURE;
            }
        };

        let mut table = SwapEntry::to_table(&entries);
        table.headings_set(!args.noheadings);
        let _ = print_table(&table, &mut io::stdout().lock());
        return ExitCode::SUCCESS;
    }

    // Build flags
    let mut flags: libc::c_int = 0;
    if let Some(ref policy) = args.discard {
        flags |= SWAP_FLAG_DISCARD;
        match policy.as_str() {
            "" => flags |= SWAP_FLAG_DISCARD_ONCE | SWAP_FLAG_DISCARD_PAGES,
            "once" => flags |= SWAP_FLAG_DISCARD_ONCE,
            "pages" => flags |= SWAP_FLAG_DISCARD_PAGES,
            other => {
                eprintln!("swapon: unknown discard policy: {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    if let Some(prio) = args.priority {
        flags |= SWAP_FLAG_PREFER | (prio & 0x7fff);
    }

    let devices = if args.all {
        read_fstab_swap_devices()
    } else if args.devices.is_empty() {
        eprintln!("swapon: no device specified. Try 'swapon --help'");
        return ExitCode::FAILURE;
    } else {
        args.devices.clone()
    };

    let mut failed = false;
    for device in &devices {
        if args.ifexists && !std::path::Path::new(device).exists() {
            continue;
        }
        if args.all && is_swap_active(device) {
            continue;
        }
        if args.verbose {
            eprintln!("swapon: enabling {device}");
        }
        if let Err(e) = do_swapon(device, flags) {
            eprintln!("swapon: {device}: {e}");
            failed = true;
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
