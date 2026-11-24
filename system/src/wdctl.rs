//! Show hardware watchdog status.
//!
//! Reads identity, timeouts, and feature and status flags from one or
//! more `/dev/watchdog*` devices via the `WDIOC_*` ioctls and
//! prints them. Closes the device with a magic-close write so the
//! kernel doesn't fire the watchdog when wdctl exits.

use clap::{ArgAction, Parser};
use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};
use std::{
    ffi::CString,
    fs::File,
    io::Write,
    mem::MaybeUninit,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    process::ExitCode,
};

pub const MAN: ManContent = ManContent::empty().section(8);

const DEFAULT_DEVICE: &str = "/dev/watchdog0";

// `<linux/watchdog.h>` is uapi, not in libc.

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct WatchdogInfo {
    options: u32,
    firmware_version: u32,
    identity: [u8; 32],
}

const WATCHDOG_IOCTL_BASE: u32 = b'W' as u32;

// `_IOR` / `_IOWR` Linux ioctl encoding:
//   31..30 dir | 29..16 size | 15..8 type | 7..0 nr
const fn ior(typ: u32, nr: u32, size: u32) -> libc::c_ulong {
    ((2u32 << 30) | (size << 16) | (typ << 8) | nr) as libc::c_ulong
}

const WDIOC_GETSUPPORT: libc::c_ulong = ior(
    WATCHDOG_IOCTL_BASE,
    0,
    std::mem::size_of::<WatchdogInfo>() as u32,
);
const WDIOC_GETSTATUS: libc::c_ulong = ior(WATCHDOG_IOCTL_BASE, 1, 4);
const WDIOC_GETBOOTSTATUS: libc::c_ulong = ior(WATCHDOG_IOCTL_BASE, 2, 4);
const WDIOC_GETTIMEOUT: libc::c_ulong = ior(WATCHDOG_IOCTL_BASE, 7, 4);
const WDIOC_GETPRETIMEOUT: libc::c_ulong = ior(WATCHDOG_IOCTL_BASE, 9, 4);
const WDIOC_GETTIMELEFT: libc::c_ulong = ior(WATCHDOG_IOCTL_BASE, 10, 4);

const WDIOF_OVERHEAT: u32 = 0x0001;
const WDIOF_FANFAULT: u32 = 0x0002;
const WDIOF_EXTERN1: u32 = 0x0004;
const WDIOF_EXTERN2: u32 = 0x0008;
const WDIOF_POWERUNDER: u32 = 0x0010;
const WDIOF_CARDRESET: u32 = 0x0020;
const WDIOF_POWEROVER: u32 = 0x0040;
const WDIOF_SETTIMEOUT: u32 = 0x0080;
const WDIOF_MAGICCLOSE: u32 = 0x0100;
const WDIOF_PRETIMEOUT: u32 = 0x0200;
const WDIOF_ALARMONLY: u32 = 0x0400;
const WDIOF_KEEPALIVEPING: u32 = 0x8000;

/// Mapping table for the bit-named flags. Order matches upstream's
/// output so default rendering lines up with what wdctl(8) prints.
const FLAG_TABLE: &[(u32, &str, &str)] = &[
    (
        WDIOF_KEEPALIVEPING,
        "KEEPALIVEPING",
        "Keep alive ping reply",
    ),
    (WDIOF_SETTIMEOUT, "SETTIMEOUT", "Set timeout (in seconds)"),
    (WDIOF_MAGICCLOSE, "MAGICCLOSE", "Supports magic close char"),
    (WDIOF_PRETIMEOUT, "PRETIMEOUT", "Pretimeout (in seconds)"),
    (WDIOF_ALARMONLY, "ALARMONLY", "Not a reboot, just an alarm"),
    (WDIOF_OVERHEAT, "OVERHEAT", "Reset due to CPU overheat"),
    (WDIOF_FANFAULT, "FANFAULT", "Fan failed"),
    (WDIOF_EXTERN1, "EXTERN1", "External relay 1"),
    (WDIOF_EXTERN2, "EXTERN2", "External relay 2"),
    (WDIOF_POWERUNDER, "POWERUNDER", "Power bad/power fault"),
    (
        WDIOF_CARDRESET,
        "CARDRESET",
        "Card previously reset the CPU",
    ),
    (WDIOF_POWEROVER, "POWEROVER", "Power over voltage"),
];

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "wdctl", version, about = "Show hardware watchdog status")]
pub struct Args {
    /// Watchdog device path (default: /dev/watchdog0)
    #[arg(short = 'd', long = "device", value_name = "PATH")]
    pub device: Option<String>,

    /// Don't print the identity and device line
    #[arg(short = 'I', long = "noident", action = ArgAction::SetTrue)]
    pub noident: bool,

    /// Don't print the table of feature and status flags
    #[arg(short = 'F', long = "noflags", action = ArgAction::SetTrue)]
    pub noflags: bool,

    /// Don't print the timeout block
    #[arg(short = 'T', long = "notimeouts", action = ArgAction::SetTrue)]
    pub notimeouts: bool,

    /// Print only the flags table (suppress identity and timeouts)
    #[arg(short = 'x', long = "flags-only", action = ArgAction::SetTrue)]
    pub flags_only: bool,

    /// Don't print the flags-table header row
    #[arg(short = 'n', long = "noheadings", action = ArgAction::SetTrue)]
    pub noheadings: bool,

    /// Devices to query. With none, /dev/watchdog0 is used.
    pub devices: Vec<String>,
}

pub fn run(args: Args) -> ExitCode {
    let devices = pick_devices(&args);
    let mut any_failed = false;
    for (i, dev) in devices.iter().enumerate() {
        if i > 0 {
            println!();
        }
        if !show_device(&args, dev) {
            any_failed = true;
        }
    }
    if any_failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn pick_devices(args: &Args) -> Vec<String> {
    if !args.devices.is_empty() {
        args.devices.clone()
    } else if let Some(d) = &args.device {
        vec![d.clone()]
    } else {
        vec![DEFAULT_DEVICE.to_string()]
    }
}

fn show_device(args: &Args, path: &str) -> bool {
    let fd = match open_watchdog(path) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("wdctl: {path}: {e}");
            return false;
        }
    };

    let info = read_info(fd.as_raw_fd());
    let status = read_int(fd.as_raw_fd(), WDIOC_GETSTATUS).ok();
    let boot_status = read_int(fd.as_raw_fd(), WDIOC_GETBOOTSTATUS).ok();
    let timeout = read_int(fd.as_raw_fd(), WDIOC_GETTIMEOUT).ok();
    let pretimeout = read_int(fd.as_raw_fd(), WDIOC_GETPRETIMEOUT).ok();
    let timeleft = read_int(fd.as_raw_fd(), WDIOC_GETTIMELEFT).ok();

    let print_ident = !args.noident && !args.flags_only;
    let print_timeouts = !args.notimeouts && !args.flags_only;
    let print_flags = !args.noflags;

    if print_ident {
        println!("Device:        {path}");
        if let Ok(i) = &info {
            println!(
                "Identity:      {} [version {}]",
                identity_str(&i.identity),
                i.firmware_version
            );
        }
    }
    if print_timeouts {
        if let Some(t) = timeout {
            println!("Timeout:       {t} seconds");
        }
        if let Some(p) = pretimeout {
            println!("Pre-timeout:   {p} seconds");
        }
        if let Some(l) = timeleft {
            println!("Timeleft:      {l} seconds");
        }
    }
    if print_flags && let Ok(i) = &info {
        print_flag_table(
            i.options,
            status.unwrap_or(0) as u32,
            boot_status.unwrap_or(0) as u32,
            args.noheadings,
        );
    }

    write_magic_close(fd);
    true
}

/// Open the watchdog device. Some drivers arm the timer just from
/// the open(). The magic-close write on drop is what tells the
/// kernel not to fire when wdctl exits.
fn open_watchdog(path: &str) -> std::io::Result<OwnedFd> {
    let cpath = CString::new(path).map_err(std::io::Error::other)?;
    let fd =
        unsafe { libc::open(cpath.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn read_info(fd: i32) -> std::io::Result<WatchdogInfo> {
    let mut info = MaybeUninit::<WatchdogInfo>::uninit();
    let rc = unsafe {
        libc::ioctl(fd, WDIOC_GETSUPPORT as libc::Ioctl, info.as_mut_ptr())
    };
    if rc < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { info.assume_init() })
}

fn read_int(fd: i32, op: libc::c_ulong) -> std::io::Result<i32> {
    let mut v: i32 = 0;
    let rc = unsafe { libc::ioctl(fd, op as libc::Ioctl, &mut v as *mut i32) };
    if rc < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(v)
}

/// The watchdog uapi lets a userspace process release the device
/// without firing the timer by writing the magic 'V' character
/// before close. Drivers that don't honor magic-close will fire
/// anyway. Not much we can do about those.
fn write_magic_close(fd: OwnedFd) {
    let mut f = File::from(fd);
    let _ = f.write_all(b"V");
}

fn identity_str(buf: &[u8; 32]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

fn print_flag_table(
    options: u32,
    status: u32,
    boot_status: u32,
    no_headings: bool,
) {
    if !no_headings {
        // Column widths: name 18 and a space, desc 32 and a space,
        // status 6 and a space, then boot. The trailing strings are
        // short enough to write as a literal.
        println!(
            "FLAG               DESCRIPTION                      STATUS BOOT-STATUS"
        );
    }
    for (bit, name, desc) in FLAG_TABLE {
        if options & bit != 0 {
            let s = (status & bit != 0) as u32;
            let b = (boot_status & bit != 0) as u32;
            println!("{name:<18} {desc:<32} {s:<6} {b}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ioctl_numbers_match_kernel_uapi() {
        // Spot-check against the values printed by `WDIOC_GETSUPPORT`
        // resolution at compile time on Linux.
        assert_eq!(WDIOC_GETSUPPORT, 0x80285700);
        assert_eq!(WDIOC_GETSTATUS, 0x80045701);
        assert_eq!(WDIOC_GETBOOTSTATUS, 0x80045702);
        assert_eq!(WDIOC_GETTIMEOUT, 0x80045707);
        assert_eq!(WDIOC_GETPRETIMEOUT, 0x80045709);
        assert_eq!(WDIOC_GETTIMELEFT, 0x8004570a);
    }

    #[test]
    fn identity_from_zero_padded_buffer() {
        let mut buf = [0u8; 32];
        let label = b"Software Watchdog";
        buf[..label.len()].copy_from_slice(label);
        // Bytes after label are NUL, and `identity_str` should stop at
        // the first NUL.
        assert_eq!(identity_str(&buf), "Software Watchdog");
    }

    #[test]
    fn identity_with_no_nul_uses_full_buffer() {
        let buf = [b'A'; 32];
        assert_eq!(identity_str(&buf), "A".repeat(32));
    }

    #[test]
    fn pick_devices_defaults_to_watchdog0() {
        let a = Args {
            device: None,
            noident: false,
            noflags: false,
            notimeouts: false,
            flags_only: false,
            noheadings: false,
            devices: vec![],
        };
        assert_eq!(pick_devices(&a), vec![DEFAULT_DEVICE.to_string()]);
    }

    #[test]
    fn pick_devices_positional_wins_over_dash_d() {
        let a = Args {
            device: Some("/dev/watchdog1".into()),
            noident: false,
            noflags: false,
            notimeouts: false,
            flags_only: false,
            noheadings: false,
            devices: vec!["/dev/watchdog2".into(), "/dev/watchdog3".into()],
        };
        assert_eq!(
            pick_devices(&a),
            vec!["/dev/watchdog2".to_string(), "/dev/watchdog3".to_string()]
        );
    }
}
