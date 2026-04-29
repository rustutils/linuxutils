use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs::{self, File},
    io::{self, BufRead},
    process::{Command, ExitCode},
};

const DEFAULT_DEVICE: &str = "/dev/cdrom";

// CD-ROM ioctls
const CDROMEJECT: libc::c_ulong = 0x5309;
const CDROMCLOSETRAY: libc::c_ulong = 0x5319;
const CDROM_LOCKDOOR: libc::c_ulong = 0x5329;
const CDROM_DRIVE_STATUS: libc::c_ulong = 0x5326;

// Drive status results
const CDS_TRAY_OPEN: i32 = 2;

// Floppy ioctl
const FDEJECT: libc::c_ulong = 0x025a;

// Tape ioctl
const MTIOCTOP: libc::c_ulong = 0x4d01;
const MTOFFL: i16 = 7;

// SCSI
const SG_IO: libc::c_ulong = 0x2285;
const SG_DXFER_NONE: i32 = -1;

#[repr(C)]
struct MtOp {
    mt_op: i16,
    mt_count: i32,
}

#[repr(C)]
struct SgIoHdr {
    interface_id: i32,
    dxfer_direction: i32,
    cmd_len: u8,
    mx_sb_len: u8,
    iovec_count: u16,
    dxfer_len: u32,
    dxferp: *mut libc::c_void,
    cmdp: *const u8,
    sbp: *mut u8,
    timeout: u32,
    flags: u32,
    pack_id: i32,
    usr_ptr: *mut libc::c_void,
    status: u8,
    masked_status: u8,
    msg_status: u8,
    sb_len_wr: u8,
    host_status: u16,
    driver_status: u16,
    resid: i32,
    duration: u32,
    info: u32,
}

#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "eject", about = "Eject removable media")]
pub struct Args {
    /// Display default device name
    #[arg(short = 'd', long = "default")]
    default: bool,

    /// Force eject, skip device type check
    #[arg(short = 'F', long)]
    force: bool,

    /// Use floppy eject method
    #[arg(short = 'f', long)]
    floppy: bool,

    /// Lock or unlock hardware eject button (on|off)
    #[arg(short = 'i', long = "manualeject")]
    manualeject: Option<String>,

    /// Don't unmount other partitions on the device
    #[arg(short = 'M', long = "no-partitions-unmount")]
    no_partitions_unmount: bool,

    /// Don't unmount the device before ejecting
    #[arg(short = 'm', long = "no-unmount")]
    no_unmount: bool,

    /// Show device but take no action
    #[arg(short = 'n', long)]
    noop: bool,

    /// Use tape drive offline command
    #[arg(short = 'q', long)]
    tape: bool,

    /// Use CD-ROM eject command
    #[arg(short = 'r', long)]
    cdrom: bool,

    /// Use SCSI eject commands
    #[arg(short = 's', long)]
    scsi: bool,

    /// Toggle tray open/close
    #[arg(short = 'T', long)]
    traytoggle: bool,

    /// Close CD-ROM tray
    #[arg(short = 't', long)]
    trayclose: bool,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Device or mountpoint to eject
    device: Option<String>,
}

fn resolve_device(device: &str) -> String {
    // If it's a mountpoint, find the device from /proc/mounts
    if let Ok(file) = File::open("/proc/mounts") {
        for line in io::BufReader::new(file).lines().map_while(Result::ok) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 2 && fields[1] == device {
                return fields[0].to_string();
            }
        }
    }
    // Follow symlinks
    fs::canonicalize(device)
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| device.to_string())
}

fn find_mounts(device: &str) -> Vec<String> {
    let mut mounts = Vec::new();
    let Ok(file) = File::open("/proc/mounts") else {
        return mounts;
    };
    for line in io::BufReader::new(file).lines().map_while(Result::ok) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 2 && fields[0] == device {
            mounts.push(fields[1].to_string());
        }
    }
    mounts
}

fn unmount(mountpoint: &str, verbose: bool) -> io::Result<()> {
    if verbose {
        eprintln!("eject: unmounting {mountpoint}");
    }
    let status = Command::new("umount").arg(mountpoint).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("umount {mountpoint} failed")))
    }
}

fn eject_cdrom(fd: i32) -> io::Result<()> {
    if unsafe { libc::ioctl(fd, CDROMEJECT) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn eject_scsi(fd: i32) -> io::Result<()> {
    // First unlock
    let unlock_cmd: [u8; 6] = [0x1e, 0, 0, 0, 0, 0];
    let mut sense: [u8; 32] = [0; 32];

    let mut hdr: SgIoHdr = unsafe { std::mem::zeroed() };
    hdr.interface_id = b'S' as i32;
    hdr.dxfer_direction = SG_DXFER_NONE;
    hdr.cmd_len = 6;
    hdr.mx_sb_len = sense.len() as u8;
    hdr.cmdp = unlock_cmd.as_ptr();
    hdr.sbp = sense.as_mut_ptr();
    hdr.timeout = 10000;

    if unsafe { libc::ioctl(fd, SG_IO, &mut hdr) } < 0 {
        return Err(io::Error::last_os_error());
    }

    // Then eject: START_STOP with LoEj=1, Start=0
    let eject_cmd: [u8; 6] = [0x1b, 0, 0, 0, 0x02, 0];
    let mut sense2: [u8; 32] = [0; 32];

    let mut hdr2: SgIoHdr = unsafe { std::mem::zeroed() };
    hdr2.interface_id = b'S' as i32;
    hdr2.dxfer_direction = SG_DXFER_NONE;
    hdr2.cmd_len = 6;
    hdr2.mx_sb_len = sense2.len() as u8;
    hdr2.cmdp = eject_cmd.as_ptr();
    hdr2.sbp = sense2.as_mut_ptr();
    hdr2.timeout = 10000;

    if unsafe { libc::ioctl(fd, SG_IO, &mut hdr2) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn eject_floppy(fd: i32) -> io::Result<()> {
    if unsafe { libc::ioctl(fd, FDEJECT) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn eject_tape(fd: i32) -> io::Result<()> {
    let op = MtOp {
        mt_op: MTOFFL,
        mt_count: 0,
    };
    if unsafe { libc::ioctl(fd, MTIOCTOP, &op) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn close_tray(fd: i32) -> io::Result<()> {
    if unsafe { libc::ioctl(fd, CDROMCLOSETRAY) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn toggle_tray(fd: i32) -> io::Result<()> {
    let status = unsafe { libc::ioctl(fd, CDROM_DRIVE_STATUS, 0) };
    if status < 0 {
        return Err(io::Error::last_os_error());
    }
    if status == CDS_TRAY_OPEN {
        close_tray(fd)
    } else {
        eject_cdrom(fd)
    }
}

fn lock_door(fd: i32, lock: bool) -> io::Result<()> {
    let arg: i32 = if lock { 1 } else { 0 };
    if unsafe { libc::ioctl(fd, CDROM_LOCKDOOR, arg) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn run(args: Args) -> ExitCode {
    if args.default {
        println!("{DEFAULT_DEVICE}");
        return ExitCode::SUCCESS;
    }

    let device = args.device.as_deref().unwrap_or(DEFAULT_DEVICE);
    let device = resolve_device(device);

    if args.noop {
        println!("{device}");
        return ExitCode::SUCCESS;
    }

    if args.verbose {
        eprintln!("eject: device is '{device}'");
    }

    // Unmount if needed
    if !args.no_unmount
        && args.manualeject.is_none()
        && !args.trayclose
        && !args.traytoggle
    {
        let mounts = find_mounts(&device);
        for mp in &mounts {
            if let Err(e) = unmount(mp, args.verbose) {
                eprintln!("eject: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    // Open device
    let flags = if args.force {
        libc::O_RDONLY | libc::O_NONBLOCK
    } else {
        libc::O_RDONLY | libc::O_NONBLOCK | libc::O_EXCL
    };
    let dev_cstr = match std::ffi::CString::new(device.as_str()) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("eject: invalid device path");
            return ExitCode::FAILURE;
        }
    };
    let fd = unsafe { libc::open(dev_cstr.as_ptr(), flags) };
    if fd < 0 {
        eprintln!(
            "eject: failed to open '{device}': {}",
            io::Error::last_os_error()
        );
        return ExitCode::FAILURE;
    }

    let result = if let Some(ref val) = args.manualeject {
        match val.as_str() {
            "on" | "1" => {
                if args.verbose {
                    eprintln!("eject: locking door");
                }
                lock_door(fd, true)
            }
            "off" | "0" => {
                if args.verbose {
                    eprintln!("eject: unlocking door");
                }
                lock_door(fd, false)
            }
            other => {
                eprintln!(
                    "eject: invalid --manualeject value '{other}' (expected on|off)"
                );
                unsafe { libc::close(fd) };
                return ExitCode::FAILURE;
            }
        }
    } else if args.trayclose {
        if args.verbose {
            eprintln!("eject: closing tray");
        }
        close_tray(fd)
    } else if args.traytoggle {
        if args.verbose {
            eprintln!("eject: toggling tray");
        }
        toggle_tray(fd)
    } else {
        // Eject: try methods in order, or use specific method
        let specific = args.cdrom || args.scsi || args.floppy || args.tape;

        let mut ok = false;

        if !ok && (args.cdrom || !specific) {
            if args.verbose {
                eprintln!("eject: trying CD-ROM eject");
            }
            if eject_cdrom(fd).is_ok() {
                ok = true;
            }
        }
        if !ok && (args.scsi || !specific) {
            if args.verbose {
                eprintln!("eject: trying SCSI eject");
            }
            if eject_scsi(fd).is_ok() {
                ok = true;
            }
        }
        if !ok && (args.floppy || !specific) {
            if args.verbose {
                eprintln!("eject: trying floppy eject");
            }
            if eject_floppy(fd).is_ok() {
                ok = true;
            }
        }
        if !ok && (args.tape || !specific) {
            if args.verbose {
                eprintln!("eject: trying tape eject");
            }
            if eject_tape(fd).is_ok() {
                ok = true;
            }
        }

        if ok {
            Ok(())
        } else {
            Err(io::Error::other("all eject methods failed"))
        }
    };

    unsafe { libc::close(fd) };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("eject: {device}: {e}");
            ExitCode::FAILURE
        }
    }
}
