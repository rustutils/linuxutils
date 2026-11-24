use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty().section(8);

use clap::Parser;
use cols::{Cols, print_table};
use std::{
    fs::{self, File},
    io,
    os::unix::io::AsRawFd,
    path::Path,
    process::ExitCode,
};

const LOOP_SET_FD: libc::c_ulong = 0x4C00;
const LOOP_CLR_FD: libc::c_ulong = 0x4C01;
const LOOP_SET_STATUS64: libc::c_ulong = 0x4C04;
const LOOP_GET_STATUS64: libc::c_ulong = 0x4C05;
const LOOP_SET_CAPACITY: libc::c_ulong = 0x4C07;
const LOOP_CTL_GET_FREE: libc::c_ulong = 0x4C82;

const LO_FLAGS_READ_ONLY: u32 = 1;
const _LO_FLAGS_AUTOCLEAR: u32 = 4;
const LO_FLAGS_PARTSCAN: u32 = 8;

#[repr(C)]
struct LoopInfo64 {
    lo_device: u64,
    lo_inode: u64,
    lo_rdevice: u64,
    lo_offset: u64,
    lo_sizelimit: u64,
    lo_number: u32,
    lo_encrypt_type: u32,
    lo_encrypt_key_size: u32,
    lo_flags: u32,
    lo_file_name: [u8; 64],
    lo_crypt_name: [u8; 64],
    lo_encrypt_key: [u8; 32],
    lo_init: [u64; 2],
}

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "losetup", about = "Set up and control loop devices")]
pub struct Args {
    /// List all used devices
    #[arg(short = 'a', long)]
    all: bool,

    /// Detach one or more devices
    #[arg(short = 'd', long)]
    detach: bool,

    /// Detach all used devices
    #[arg(short = 'D', long = "detach-all")]
    detach_all: bool,

    /// Find first unused device
    #[arg(short = 'f', long)]
    find: bool,

    /// Resize loop device (reread backing file size)
    #[arg(short = 'c', long = "set-capacity")]
    set_capacity: bool,

    /// List devices associated with a backing file
    #[arg(short = 'j', long)]
    associated: Option<String>,

    /// Byte offset into backing file
    #[arg(short = 'o', long)]
    offset: Option<u64>,

    /// Limit device size in bytes
    #[arg(long)]
    sizelimit: Option<u64>,

    /// Force kernel partition table scan
    #[arg(short = 'P', long)]
    partscan: bool,

    /// Set up read-only
    #[arg(short = 'r', long = "read-only")]
    read_only: bool,

    /// Print device name after setup (with -f)
    #[arg(long)]
    show: bool,

    /// Tabular list format
    #[arg(short = 'l', long)]
    list: bool,

    /// Suppress headers in list output
    #[arg(short = 'n', long)]
    noheadings: bool,

    /// Verbose mode
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Loop devices, backing files, or both
    #[arg(trailing_var_arg = true)]
    positional: Vec<String>,
}

#[derive(Cols)]
struct LoopDeviceInfo {
    #[column(header = "NAME")]
    name: String,

    #[column(right, header = "SIZELIMIT")]
    sizelimit: u64,

    #[column(right, header = "OFFSET")]
    offset: u64,

    #[column(right, header = "AUTOCLEAR")]
    autoclear: u8,

    #[column(right, header = "PARTSCAN")]
    partscan: u8,

    #[column(right, header = "RO")]
    read_only: u8,

    #[column(header = "BACK-FILE")]
    backing_file: String,

    #[column(right, header = "DIO")]
    dio: u8,
}

fn sysfs_read(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn list_loop_devices() -> Vec<LoopDeviceInfo> {
    let mut devices = Vec::new();

    let Ok(entries) = fs::read_dir("/sys/block") else {
        return devices;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("loop") {
            continue;
        }

        let loop_dir = format!("/sys/block/{name_str}/loop");
        if !Path::new(&loop_dir).exists() {
            continue;
        }

        let Some(backing_file) =
            sysfs_read(&format!("{loop_dir}/backing_file"))
        else {
            continue;
        };

        let bool_val = |path: &str| -> u8 {
            sysfs_read(path)
                .map(|s| if s == "1" { 1 } else { 0 })
                .unwrap_or(0)
        };

        devices.push(LoopDeviceInfo {
            name: format!("/dev/{name_str}"),
            sizelimit: sysfs_read(&format!("{loop_dir}/sizelimit"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            offset: sysfs_read(&format!("{loop_dir}/offset"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            autoclear: bool_val(&format!("{loop_dir}/autoclear")),
            partscan: bool_val(&format!("{loop_dir}/partscan")),
            read_only: bool_val(&format!("/sys/block/{name_str}/ro")),
            backing_file,
            dio: bool_val(&format!("{loop_dir}/dio")),
        });
    }

    devices.sort_by(|a, b| a.name.cmp(&b.name));
    devices
}

fn print_devices(devices: &[LoopDeviceInfo], noheadings: bool) {
    let mut table = LoopDeviceInfo::to_table(devices);
    table.headings_set(!noheadings);
    let _ = print_table(&table, &mut io::stdout().lock());
}

fn print_old_style(devices: &[LoopDeviceInfo]) {
    for d in devices {
        let mut extra = String::new();
        if d.offset > 0 {
            extra.push_str(&format!(", offset {}", d.offset));
        }
        if d.sizelimit > 0 {
            extra.push_str(&format!(", sizelimit {}", d.sizelimit));
        }
        println!("{}: ({}){extra}", d.name, d.backing_file);
    }
}

fn find_free_device() -> io::Result<String> {
    let ctl = File::open("/dev/loop-control")?;
    let nr = unsafe {
        libc::ioctl(ctl.as_raw_fd(), LOOP_CTL_GET_FREE as libc::Ioctl)
    };
    if nr < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(format!("/dev/loop{nr}"))
}

fn setup_loop(
    device: &str,
    file: &str,
    offset: u64,
    sizelimit: u64,
    flags: u32,
) -> io::Result<()> {
    let backing = if flags & LO_FLAGS_READ_ONLY != 0 {
        File::open(file)?
    } else {
        File::options().read(true).write(true).open(file)?
    };

    let loop_dev = File::options().read(true).write(true).open(device)?;

    let ret = unsafe {
        libc::ioctl(
            loop_dev.as_raw_fd(),
            LOOP_SET_FD as libc::Ioctl,
            backing.as_raw_fd(),
        )
    };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut info = unsafe { std::mem::zeroed::<LoopInfo64>() };
    info.lo_offset = offset;
    info.lo_sizelimit = sizelimit;
    info.lo_flags = flags;

    let file_bytes = file.as_bytes();
    let copy_len = file_bytes.len().min(63);
    info.lo_file_name[..copy_len].copy_from_slice(&file_bytes[..copy_len]);

    let ret = unsafe {
        libc::ioctl(
            loop_dev.as_raw_fd(),
            LOOP_SET_STATUS64 as libc::Ioctl,
            &info,
        )
    };
    if ret < 0 {
        let err = io::Error::last_os_error();
        unsafe {
            libc::ioctl(loop_dev.as_raw_fd(), LOOP_CLR_FD as libc::Ioctl, 0)
        };
        return Err(err);
    }

    Ok(())
}

fn detach_loop(device: &str) -> io::Result<()> {
    let f = File::open(device)?;
    let ret =
        unsafe { libc::ioctl(f.as_raw_fd(), LOOP_CLR_FD as libc::Ioctl, 0) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn set_capacity(device: &str) -> io::Result<()> {
    let f = File::open(device)?;
    let ret = unsafe {
        libc::ioctl(f.as_raw_fd(), LOOP_SET_CAPACITY as libc::Ioctl, 0)
    };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn show_device(device: &str) -> ExitCode {
    let Ok(f) = File::open(device) else {
        eprintln!("losetup: {device}: failed to open");
        return ExitCode::from(2);
    };

    let mut info = unsafe { std::mem::zeroed::<LoopInfo64>() };
    let ret = unsafe {
        libc::ioctl(f.as_raw_fd(), LOOP_GET_STATUS64 as libc::Ioctl, &mut info)
    };
    if ret < 0 {
        let e = io::Error::last_os_error();
        if e.raw_os_error() == Some(libc::ENXIO) {
            eprintln!("losetup: {device}: not a configured loop device");
            return ExitCode::FAILURE;
        }
        eprintln!("losetup: {device}: {e}");
        return ExitCode::from(2);
    }

    let file_name = extract_string(&info.lo_file_name);
    let mut extra = String::new();
    if info.lo_offset > 0 {
        extra.push_str(&format!(", offset {}", info.lo_offset));
    }
    if info.lo_sizelimit > 0 {
        extra.push_str(&format!(", sizelimit {}", info.lo_sizelimit));
    }
    println!("{device}: ({file_name}){extra}");
    ExitCode::SUCCESS
}

fn extract_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

pub fn run(args: Args) -> ExitCode {
    if args.detach_all {
        let devices = list_loop_devices();
        let mut failed = false;
        for d in &devices {
            if let Err(e) = detach_loop(&d.name) {
                eprintln!("losetup: {}: {e}", d.name);
                failed = true;
            }
        }
        return if failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    if args.detach {
        if args.positional.is_empty() {
            eprintln!("losetup: --detach requires at least one device");
            return ExitCode::FAILURE;
        }
        let mut failed = false;
        for dev in &args.positional {
            if let Err(e) = detach_loop(dev) {
                eprintln!("losetup: {dev}: {e}");
                failed = true;
            }
        }
        return if failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    if args.set_capacity {
        let dev = match args.positional.first() {
            Some(d) => d,
            None => {
                eprintln!("losetup: --set-capacity requires a device");
                return ExitCode::FAILURE;
            }
        };
        return match set_capacity(dev) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("losetup: {dev}: {e}");
                ExitCode::FAILURE
            }
        };
    }

    if let Some(ref file) = args.associated {
        let canonical = fs::canonicalize(file)
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| file.clone());

        let devices: Vec<_> = list_loop_devices()
            .into_iter()
            .filter(|d| {
                d.backing_file == canonical
                    || d.backing_file == *file
                    || d.backing_file.trim_end() == canonical
            })
            .collect();

        print_old_style(&devices);
        return ExitCode::SUCCESS;
    }

    if args.find {
        let dev = match find_free_device() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("losetup: failed to find a free loop device: {e}");
                return ExitCode::FAILURE;
            }
        };

        if let Some(file) = args.positional.first() {
            let mut flags = 0u32;
            if args.read_only {
                flags |= LO_FLAGS_READ_ONLY;
            }
            if args.partscan {
                flags |= LO_FLAGS_PARTSCAN;
            }

            if let Err(e) = setup_loop(
                &dev,
                file,
                args.offset.unwrap_or(0),
                args.sizelimit.unwrap_or(0),
                flags,
            ) {
                eprintln!("losetup: {dev}: {e}");
                return ExitCode::FAILURE;
            }

            if args.show {
                println!("{dev}");
            }
        } else {
            println!("{dev}");
        }

        return ExitCode::SUCCESS;
    }

    // List mode: no args = tabular, -a alone = old-style, -l or -a -l = tabular
    if args.positional.is_empty()
        && (args.all
            || args.list
            || (!args.detach && !args.find && !args.set_capacity))
    {
        let devices = list_loop_devices();
        if args.all && !args.list {
            print_old_style(&devices);
        } else {
            print_devices(&devices, args.noheadings);
        }
        return ExitCode::SUCCESS;
    }

    // Show or setup a specific device
    if args.positional.len() == 1 {
        return show_device(&args.positional[0]);
    }

    if args.positional.len() == 2 {
        let dev = &args.positional[0];
        let file = &args.positional[1];

        let mut flags = 0u32;
        if args.read_only {
            flags |= LO_FLAGS_READ_ONLY;
        }
        if args.partscan {
            flags |= LO_FLAGS_PARTSCAN;
        }

        return match setup_loop(
            dev,
            file,
            args.offset.unwrap_or(0),
            args.sizelimit.unwrap_or(0),
            flags,
        ) {
            Ok(()) => {
                if args.verbose {
                    println!("{dev}: set up with {file}");
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("losetup: {dev}: {e}");
                ExitCode::FAILURE
            }
        };
    }

    eprintln!("losetup: bad usage, try 'losetup --help'");
    ExitCode::FAILURE
}
