use clap::Parser;
use std::{
    ffi::CString,
    fs::File,
    io::{self, BufRead},
    process::ExitCode,
};

/// Disable devices and files for paging and swapping.
///
/// Calls swapoff(2) to deactivate swap areas. The kernel pages all
/// swap contents back to RAM before deactivating, which may take
/// significant time if the swap is heavily used.
#[derive(Parser)]
#[command(
    name = "swapoff",
    about = "Disable devices and files for paging and swapping"
)]
pub struct Args {
    /// Disable all active swap areas
    #[arg(short = 'a', long)]
    all: bool,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Devices or files to disable
    devices: Vec<String>,
}

fn read_active_swaps() -> Vec<String> {
    let Ok(file) = File::open("/proc/swaps") else {
        return Vec::new();
    };
    io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .filter_map(|line| {
            line.split_whitespace().next().map(|s| s.to_string())
        })
        .collect()
}

fn do_swapoff(path: &str) -> io::Result<()> {
    let cpath =
        CString::new(path).map_err(|e| io::Error::other(e.to_string()))?;
    if unsafe { libc::swapoff(cpath.as_ptr()) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn run(args: Args) -> ExitCode {
    let devices = if args.all {
        read_active_swaps()
    } else if args.devices.is_empty() {
        eprintln!("swapoff: no device specified. Try 'swapoff --help'");
        return ExitCode::from(16);
    } else {
        args.devices.clone()
    };

    let mut successes = 0;
    let mut failures = 0;

    for device in &devices {
        if args.verbose {
            eprintln!("swapoff: disabling {device}");
        }
        if let Err(e) = do_swapoff(device) {
            eprintln!("swapoff: {device}: {e}");
            failures += 1;
        } else {
            successes += 1;
        }
    }

    if failures == 0 {
        ExitCode::SUCCESS
    } else if args.all && successes == 0 {
        ExitCode::from(32)
    } else if args.all {
        ExitCode::from(64)
    } else {
        ExitCode::from(4)
    }
}
