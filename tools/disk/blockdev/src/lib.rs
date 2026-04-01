use clap::Parser;
use cols::{Cols, print_table};
use std::{
    fs::File,
    io::{self, BufRead},
    os::unix::io::AsRawFd,
    process::ExitCode,
};

// Block device ioctls
const BLKROSET: libc::c_ulong = 0x125d;
const BLKROGET: libc::c_ulong = 0x125e;
const BLKRRPART: libc::c_ulong = 0x125f;
const BLKGETSIZE: libc::c_ulong = 0x1260;
const BLKFLSBUF: libc::c_ulong = 0x1261;
const BLKRASET: libc::c_ulong = 0x1262;
const BLKRAGET: libc::c_ulong = 0x1263;
const BLKFRASET: libc::c_ulong = 0x1264;
const BLKFRAGET: libc::c_ulong = 0x1265;
const BLKSECTGET: libc::c_ulong = 0x1267;
const BLKSSZGET: libc::c_ulong = 0x1268;
const BLKBSZGET: libc::c_ulong = 0x80081270;
const BLKBSZSET: libc::c_ulong = 0x40081271;
const BLKGETSIZE64: libc::c_ulong = 0x80081272;
const BLKIOMIN: libc::c_ulong = 0x1278;
const BLKIOOPT: libc::c_ulong = 0x1279;
const BLKALIGNOFF: libc::c_ulong = 0x127a;
const BLKPBSZGET: libc::c_ulong = 0x127b;
const BLKDISCARDZEROES: libc::c_ulong = 0x127c;

/// Call block device ioctls from the command line.
///
/// Each `--getXXX` or `--setXXX` command performs a single ioctl on
/// the specified block device(s). Multiple commands and devices can be
/// given in a single invocation.
#[derive(Parser)]
#[command(
    name = "blockdev",
    about = "Call block device ioctls from the command line",
    override_usage = "blockdev [-q] [-v] command [command...] device [device...]\n       \
                      blockdev --report [device...]"
)]
pub struct Args {
    /// Commands, arguments, and device paths
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

enum Cmd {
    GetInt(libc::c_ulong, &'static str),
    GetUint(libc::c_ulong, &'static str),
    GetU64(libc::c_ulong, &'static str),
    GetUlong(libc::c_ulong, &'static str),
    GetSz,
    SetInt(libc::c_ulong, libc::c_int, &'static str),
    SetUlong(libc::c_ulong, libc::c_ulong, &'static str),
    Action(libc::c_ulong, &'static str),
}

fn ioctl_get_int(fd: i32, nr: libc::c_ulong) -> io::Result<i64> {
    let mut val: libc::c_int = 0;
    if unsafe { libc::ioctl(fd, nr, &mut val) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(val as i64)
    }
}

fn ioctl_get_uint(fd: i32, nr: libc::c_ulong) -> io::Result<i64> {
    let mut val: libc::c_uint = 0;
    if unsafe { libc::ioctl(fd, nr, &mut val) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(val as i64)
    }
}

fn ioctl_get_u64(fd: i32, nr: libc::c_ulong) -> io::Result<u64> {
    let mut val: u64 = 0;
    if unsafe { libc::ioctl(fd, nr, &mut val) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(val)
    }
}

fn ioctl_get_ulong(fd: i32, nr: libc::c_ulong) -> io::Result<i64> {
    let mut val: libc::c_ulong = 0;
    if unsafe { libc::ioctl(fd, nr, &mut val) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(val as i64)
    }
}

fn ioctl_set_int(
    fd: i32,
    nr: libc::c_ulong,
    val: libc::c_int,
) -> io::Result<()> {
    if unsafe { libc::ioctl(fd, nr, &val) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn ioctl_set_ulong(
    fd: i32,
    nr: libc::c_ulong,
    val: libc::c_ulong,
) -> io::Result<()> {
    if unsafe { libc::ioctl(fd, nr, val) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn ioctl_action(fd: i32, nr: libc::c_ulong) -> io::Result<()> {
    if unsafe { libc::ioctl(fd, nr, 0) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn execute_cmd(
    fd: i32,
    cmd: &Cmd,
    verbose: bool,
    quiet: bool,
) -> io::Result<()> {
    match cmd {
        Cmd::GetInt(nr, name) => {
            let val = ioctl_get_int(fd, *nr)?;
            if verbose {
                print!("{name}: ");
            }
            println!("{val}");
        }
        Cmd::GetUint(nr, name) => {
            let val = ioctl_get_uint(fd, *nr)?;
            if verbose {
                print!("{name}: ");
            }
            println!("{val}");
        }
        Cmd::GetU64(nr, name) => {
            let val = ioctl_get_u64(fd, *nr)?;
            if verbose {
                print!("{name}: ");
            }
            println!("{val}");
        }
        Cmd::GetUlong(nr, name) => {
            let val = ioctl_get_ulong(fd, *nr)?;
            if verbose {
                print!("{name}: ");
            }
            println!("{val}");
        }
        Cmd::GetSz => {
            let bytes = ioctl_get_u64(fd, BLKGETSIZE64)?;
            if verbose {
                print!("getsz: ");
            }
            println!("{}", bytes / 512);
        }
        Cmd::SetInt(nr, val, name) => {
            ioctl_set_int(fd, *nr, *val)?;
            if !quiet {
                eprintln!("{name} succeeded.");
            }
        }
        Cmd::SetUlong(nr, val, name) => {
            ioctl_set_ulong(fd, *nr, *val)?;
            if !quiet {
                eprintln!("{name} succeeded.");
            }
        }
        Cmd::Action(nr, name) => {
            ioctl_action(fd, *nr)?;
            if !quiet {
                eprintln!("{name} succeeded.");
            }
        }
    }
    Ok(())
}

fn get_start_sector(device: &str) -> u64 {
    let Ok(f) = File::open(device) else {
        return 0;
    };
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(f.as_raw_fd(), &mut stat) } < 0 {
        return 0;
    }
    let major = libc::major(stat.st_rdev as libc::dev_t);
    let minor = libc::minor(stat.st_rdev as libc::dev_t);
    let path = format!("/sys/dev/block/{major}:{minor}/start");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn devices_from_proc_partitions() -> Vec<String> {
    let mut devices = Vec::new();
    let Ok(file) = File::open("/proc/partitions") else {
        return devices;
    };
    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(2)
    {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 4 {
            devices.push(format!("/dev/{}", fields[3]));
        }
    }
    devices
}

#[derive(Cols)]
struct ReportRow {
    #[column(right, header = "RO")]
    ro: i64,

    #[column(right, header = "RA")]
    ra: i64,

    #[column(right, header = "SSZ")]
    ssz: i64,

    #[column(right, header = "BSZ")]
    bsz: i64,

    #[column(right, header = "StartSec")]
    start_sec: u64,

    #[column(right, header = "Size")]
    size: u64,

    #[column(header = "Device")]
    device: String,
}

fn print_report(devices: &[String]) {
    let mut rows = Vec::new();
    for device in devices {
        let Ok(f) = File::options().read(true).open(device) else {
            continue;
        };
        let fd = f.as_raw_fd();
        rows.push(ReportRow {
            ro: ioctl_get_int(fd, BLKROGET).unwrap_or(-1),
            ra: ioctl_get_ulong(fd, BLKRAGET).unwrap_or(-1),
            ssz: ioctl_get_int(fd, BLKSSZGET).unwrap_or(-1),
            bsz: ioctl_get_int(fd, BLKBSZGET).unwrap_or(-1),
            start_sec: get_start_sector(device),
            size: ioctl_get_u64(fd, BLKGETSIZE64).unwrap_or(0),
            device: device.clone(),
        });
    }
    let table = ReportRow::to_table(&rows);
    let _ = print_table(&table, &mut io::stdout().lock());
}

struct ParsedArgs {
    commands: Vec<Cmd>,
    devices: Vec<String>,
    verbose: bool,
    quiet: bool,
    report: bool,
}

fn parse_commands(raw_args: &[String]) -> Result<ParsedArgs, String> {
    let mut commands: Vec<Cmd> = Vec::new();
    let mut devices: Vec<String> = Vec::new();
    let mut verbose = false;
    let mut quiet = false;
    let mut report = false;
    let mut i = 0;

    while i < raw_args.len() {
        let arg = &raw_args[i];
        match arg.as_str() {
            "-v" | "--verbose" => verbose = true,
            "-q" => quiet = true,
            "--report" => report = true,

            "--getro" => commands.push(Cmd::GetInt(BLKROGET, "getro")),
            "--getbsz" => commands.push(Cmd::GetInt(BLKBSZGET, "getbsz")),
            "--getss" => commands.push(Cmd::GetInt(BLKSSZGET, "getss")),
            "--getpbsz" => commands.push(Cmd::GetUint(BLKPBSZGET, "getpbsz")),
            "--getsize" => commands.push(Cmd::GetUlong(BLKGETSIZE, "getsize")),
            "--getsize64" => {
                commands.push(Cmd::GetU64(BLKGETSIZE64, "getsize64"))
            }
            "--getsz" => commands.push(Cmd::GetSz),
            "--getra" => commands.push(Cmd::GetUlong(BLKRAGET, "getra")),
            "--getfra" => commands.push(Cmd::GetUlong(BLKFRAGET, "getfra")),
            "--getiomin" => commands.push(Cmd::GetUint(BLKIOMIN, "getiomin")),
            "--getioopt" => commands.push(Cmd::GetUint(BLKIOOPT, "getioopt")),
            "--getalignoff" => {
                commands.push(Cmd::GetUint(BLKALIGNOFF, "getalignoff"))
            }
            "--getmaxsect" => {
                commands.push(Cmd::GetUint(BLKSECTGET, "getmaxsect"))
            }
            "--getdiscardzeroes" => {
                commands
                    .push(Cmd::GetUint(BLKDISCARDZEROES, "getdiscardzeroes"));
            }

            "--setro" => commands.push(Cmd::SetInt(BLKROSET, 1, "setro")),
            "--setrw" => commands.push(Cmd::SetInt(BLKROSET, 0, "setrw")),
            "--setbsz" => {
                i += 1;
                let val: libc::c_int = raw_args
                    .get(i)
                    .ok_or("--setbsz requires an argument")?
                    .parse()
                    .map_err(|_| "--setbsz: invalid number")?;
                commands.push(Cmd::SetInt(BLKBSZSET, val, "setbsz"));
            }
            "--setra" => {
                i += 1;
                let val: libc::c_ulong = raw_args
                    .get(i)
                    .ok_or("--setra requires an argument")?
                    .parse()
                    .map_err(|_| "--setra: invalid number")?;
                commands.push(Cmd::SetUlong(BLKRASET, val, "setra"));
            }
            "--setfra" => {
                i += 1;
                let val: libc::c_ulong = raw_args
                    .get(i)
                    .ok_or("--setfra requires an argument")?
                    .parse()
                    .map_err(|_| "--setfra: invalid number")?;
                commands.push(Cmd::SetUlong(BLKFRASET, val, "setfra"));
            }
            "--flushbufs" => commands.push(Cmd::Action(BLKFLSBUF, "flushbufs")),
            "--rereadpt" => commands.push(Cmd::Action(BLKRRPART, "rereadpt")),

            other if other.starts_with('-') => {
                return Err(format!("unknown command: {other}"));
            }
            _ => devices.push(arg.clone()),
        }
        i += 1;
    }

    Ok(ParsedArgs {
        commands,
        devices,
        verbose,
        quiet,
        report,
    })
}

pub fn run(args: Args) -> ExitCode {
    let ParsedArgs {
        commands,
        devices,
        verbose,
        quiet,
        report,
    } = match parse_commands(&args.args) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("blockdev: {e}");
            return ExitCode::FAILURE;
        }
    };

    if report {
        let devs = if devices.is_empty() {
            devices_from_proc_partitions()
        } else {
            devices
        };
        print_report(&devs);
        return ExitCode::SUCCESS;
    }

    if commands.is_empty() {
        eprintln!("blockdev: no command given, try 'blockdev --help'");
        return ExitCode::FAILURE;
    }

    if devices.is_empty() {
        eprintln!("blockdev: no device specified");
        return ExitCode::FAILURE;
    }

    let mut failed = false;
    for device in &devices {
        let f = match File::options().read(true).write(true).open(device) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("blockdev: cannot open {device}: {e}");
                failed = true;
                continue;
            }
        };
        let fd = f.as_raw_fd();
        for cmd in &commands {
            if let Err(e) = execute_cmd(fd, cmd, verbose, quiet) {
                eprintln!("blockdev: {device}: {e}");
                failed = true;
            }
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
