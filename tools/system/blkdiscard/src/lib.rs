use clap::Parser;
use std::{fs::File, io, os::unix::io::AsRawFd, process::ExitCode};

const BLKGETSIZE64: libc::c_ulong = 0x80081272;
const BLKSSZGET: libc::c_ulong = 0x1268;
const BLKDISCARD: libc::c_ulong = 0x1277;
const BLKSECDISCARD: libc::c_ulong = 0x127d;
const BLKZEROOUT: libc::c_ulong = 0x127f;

/// Discard sectors on a block device.
///
/// Sends BLKDISCARD, BLKSECDISCARD, or BLKZEROOUT ioctls to tell the
/// storage device that a range of blocks is no longer in use.
#[derive(Parser)]
#[command(name = "blkdiscard", about = "Discard sectors on a block device")]
pub struct Args {
    /// Disable exclusive open mode (O_EXCL)
    #[arg(short = 'f', long)]
    force: bool,

    /// Byte offset to start discarding (default: 0)
    #[arg(short = 'o', long, default_value = "0")]
    offset: u64,

    /// Number of bytes to discard (default: to end of device)
    #[arg(short = 'l', long)]
    length: Option<u64>,

    /// Bytes to discard per ioctl call (default: all at once)
    #[arg(short = 'p', long)]
    step: Option<u64>,

    /// Suppress warnings
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Secure discard (BLKSECDISCARD)
    #[arg(short = 's', long)]
    secure: bool,

    /// Zero-fill instead of discard (BLKZEROOUT)
    #[arg(short = 'z', long)]
    zeroout: bool,

    /// Show progress
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Block device to operate on
    device: String,
}

fn get_device_size(fd: i32) -> io::Result<u64> {
    let mut size: u64 = 0;
    if unsafe { libc::ioctl(fd, BLKGETSIZE64, &mut size) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(size)
    }
}

fn get_sector_size(fd: i32) -> io::Result<u32> {
    let mut ss: libc::c_int = 0;
    if unsafe { libc::ioctl(fd, BLKSSZGET, &mut ss) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ss as u32)
    }
}

fn do_discard(
    fd: i32,
    ioctl_nr: libc::c_ulong,
    offset: u64,
    length: u64,
) -> io::Result<()> {
    let range: [u64; 2] = [offset, length];
    if unsafe { libc::ioctl(fd, ioctl_nr, &range) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn run(args: Args) -> ExitCode {
    if args.secure && args.zeroout {
        eprintln!("blkdiscard: --secure and --zeroout are mutually exclusive");
        return ExitCode::FAILURE;
    }

    let oflags = if args.force {
        libc::O_RDWR
    } else {
        libc::O_RDWR | libc::O_EXCL
    };

    let dev_cstr = match std::ffi::CString::new(args.device.as_str()) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("blkdiscard: invalid device path");
            return ExitCode::FAILURE;
        }
    };

    let fd = unsafe { libc::open(dev_cstr.as_ptr(), oflags) };
    if fd < 0 {
        eprintln!(
            "blkdiscard: cannot open '{}': {}",
            args.device,
            io::Error::last_os_error()
        );
        return ExitCode::FAILURE;
    }

    // Wrap fd for RAII close (via File)
    let file = unsafe { File::from_raw_fd(fd) };
    let fd = file.as_raw_fd();

    let dev_size = match get_device_size(fd) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("blkdiscard: cannot get device size: {e}");
            return ExitCode::FAILURE;
        }
    };

    let sector_size = get_sector_size(fd).unwrap_or(512) as u64;

    let offset = args.offset;
    let length = args.length.unwrap_or(dev_size.saturating_sub(offset));

    if !offset.is_multiple_of(sector_size) {
        eprintln!(
            "blkdiscard: offset {offset} is not aligned to sector size {sector_size}"
        );
        return ExitCode::FAILURE;
    }
    if !length.is_multiple_of(sector_size) {
        eprintln!(
            "blkdiscard: length {length} is not aligned to sector size {sector_size}"
        );
        return ExitCode::FAILURE;
    }

    let ioctl_nr = if args.secure {
        BLKSECDISCARD
    } else if args.zeroout {
        BLKZEROOUT
    } else {
        BLKDISCARD
    };

    let action = if args.secure {
        "secure discard"
    } else if args.zeroout {
        "zeroout"
    } else {
        "discard"
    };

    if args.verbose {
        eprintln!(
            "blkdiscard: {action} offset={offset} length={length} device={}",
            args.device
        );
    }

    if let Some(step) = args.step {
        let mut current = offset;
        let end = offset + length;
        while current < end {
            let chunk = step.min(end - current);
            if let Err(e) = do_discard(fd, ioctl_nr, current, chunk) {
                eprintln!(
                    "blkdiscard: {action} failed at offset {current}: {e}"
                );
                return if e.raw_os_error() == Some(libc::EOPNOTSUPP) {
                    ExitCode::from(2)
                } else {
                    ExitCode::FAILURE
                };
            }
            current += chunk;
        }
    } else if let Err(e) = do_discard(fd, ioctl_nr, offset, length) {
        eprintln!("blkdiscard: {action} failed: {e}");
        return if e.raw_os_error() == Some(libc::EOPNOTSUPP) {
            ExitCode::from(2)
        } else {
            ExitCode::FAILURE
        };
    }

    ExitCode::SUCCESS
}

use std::os::unix::io::FromRawFd;
