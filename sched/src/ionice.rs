use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::process::ExitCode;

const IOPRIO_WHO_PROCESS: i32 = 1;
const IOPRIO_WHO_PGRP: i32 = 2;
const IOPRIO_WHO_USER: i32 = 3;
const IOPRIO_CLASS_SHIFT: i32 = 13;

fn ioprio_get(which: i32, who: i32) -> Result<i32, std::io::Error> {
    let ret = unsafe { libc::syscall(libc::SYS_ioprio_get, which, who) };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(ret as i32)
    }
}

fn ioprio_set(which: i32, who: i32, ioprio: i32) -> Result<(), std::io::Error> {
    let ret =
        unsafe { libc::syscall(libc::SYS_ioprio_set, which, who, ioprio) };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn class_name(class: i32) -> &'static str {
    match class {
        0 => "none",
        1 => "realtime",
        2 => "best-effort",
        3 => "idle",
        _ => "unknown",
    }
}

/// `strerror(3)`-style description of an `io::Error`. Rust's
/// default `Display` appends `(os error 22)`, but upstream's tools
/// just print the description on its own.
fn strerror(e: &std::io::Error) -> String {
    if let Some(errno) = e.raw_os_error() {
        let p = unsafe { libc::strerror(errno) };
        if !p.is_null() {
            let cs = unsafe { std::ffi::CStr::from_ptr(p) };
            return cs.to_string_lossy().into_owned();
        }
    }
    e.to_string()
}

/// Parse a class argument. Upstream accepts any integer here and
/// only rejects truly malformed input. Values that don't map to a
/// real class still go through to `ioprio_set(2)`, which is what
/// emits the `EINVAL` the user sees.
fn parse_class(s: &str) -> Result<i32, String> {
    match s {
        "none" => Ok(0),
        "realtime" | "rt" => Ok(1),
        "best-effort" | "be" => Ok(2),
        "idle" => Ok(3),
        _ => s
            .parse::<i32>()
            .map_err(|_| format!("unknown scheduling class: '{s}'")),
    }
}

fn print_ioprio(which: i32, who: i32, label: &str) -> bool {
    match ioprio_get(which, who) {
        Ok(ioprio) => {
            let class = ioprio >> IOPRIO_CLASS_SHIFT;
            let level = ioprio & ((1 << IOPRIO_CLASS_SHIFT) - 1);
            println!("{}: prio {level}", class_name(class));
            false
        }
        Err(e) => {
            eprintln!("ionice: ioprio_get failed for {label} {who}: {e}");
            true
        }
    }
}

fn set_ioprio(
    which: i32,
    who: i32,
    class: i32,
    level: i32,
    ignore: bool,
    _label: &str,
) -> bool {
    // Upstream warns (but proceeds) for class numbers it doesn't
    // recognise, then lets `ioprio_set(2)` reject them with EINVAL.
    if !(0..=3).contains(&class) {
        eprintln!("ionice: unknown prio class {class}");
    }
    let ioprio = (class << IOPRIO_CLASS_SHIFT) | level;
    match ioprio_set(which, who, ioprio) {
        Ok(()) => false,
        Err(e) => {
            eprintln!("ionice: ioprio_set failed: {}", strerror(&e));
            !ignore
        }
    }
}

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "ionice",
    about = "Set or get process I/O scheduling class and priority",
    override_usage = "ionice [-c class] [-n level] [-t] -p PID...\n       \
                      ionice [-c class] [-n level] [-t] -P PGID...\n       \
                      ionice [-c class] [-n level] [-t] -u UID...\n       \
                      ionice [-c class] [-n level] [-t] command [args...]\n       \
                      ionice [-p PID...] [-P PGID...] [-u UID...]"
)]
pub struct Args {
    /// I/O scheduling class (0=none, 1=realtime, 2=best-effort, 3=idle)
    #[arg(short = 'c', long = "class")]
    class: Option<String>,

    /// I/O scheduling priority level (0-7, 0 is highest)
    #[arg(short = 'n', long = "classdata")]
    classdata: Option<i32>,

    /// Process IDs to query or set
    #[arg(short = 'p', long = "pid", num_args = 1..)]
    pid: Option<Vec<i32>>,

    /// Process group IDs to query or set
    #[arg(short = 'P', long = "pgid", num_args = 1..)]
    pgid: Option<Vec<i32>>,

    /// User IDs to query or set
    #[arg(short = 'u', long = "uid", num_args = 1..)]
    uid: Option<Vec<i32>>,

    /// Ignore failure to set the requested priority
    #[arg(short = 't', long = "ignore")]
    ignore: bool,

    /// Command to run with the given I/O scheduling
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

pub fn run(args: Args) -> ExitCode {
    let has_targets =
        args.pid.is_some() || args.pgid.is_some() || args.uid.is_some();
    let setting = args.class.is_some() || args.classdata.is_some();

    if setting
        && let Some(ref n) = args.classdata
        && (*n < 0 || *n > 7)
    {
        eprintln!("ionice: priority level must be between 0 and 7 (got {n})");
        return ExitCode::FAILURE;
    }

    let parsed_class = if let Some(ref c) = args.class {
        match parse_class(c) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("ionice: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };

    if !setting && !has_targets && args.command.is_empty() {
        if print_ioprio(IOPRIO_WHO_PROCESS, 0, "pid") {
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    if !setting && has_targets && args.command.is_empty() {
        let mut failed = false;
        if let Some(ref pids) = args.pid {
            for &p in pids {
                failed |= print_ioprio(IOPRIO_WHO_PROCESS, p, "pid");
            }
        }
        if let Some(ref pgids) = args.pgid {
            for &p in pgids {
                failed |= print_ioprio(IOPRIO_WHO_PGRP, p, "pgid");
            }
        }
        if let Some(ref uids) = args.uid {
            for &u in uids {
                failed |= print_ioprio(IOPRIO_WHO_USER, u, "uid");
            }
        }
        return if failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    let class = parsed_class.unwrap_or(2);
    // Upstream warns when the user provides classdata for class 0
    // (none) or class 3 (idle), since those classes ignore the
    // priority level entirely.
    if (class == 0 || class == 3) && args.classdata.is_some() {
        let label = if class == 0 { "none" } else { "idle" };
        eprintln!("ionice: ignoring given class data for {label} class");
    }
    let level = if class == 0 || class == 3 {
        0
    } else {
        args.classdata.unwrap_or(4)
    };

    if has_targets {
        let mut failed = false;
        if let Some(ref pids) = args.pid {
            for &p in pids {
                failed |= set_ioprio(
                    IOPRIO_WHO_PROCESS,
                    p,
                    class,
                    level,
                    args.ignore,
                    "pid",
                );
            }
        }
        if let Some(ref pgids) = args.pgid {
            for &p in pgids {
                failed |= set_ioprio(
                    IOPRIO_WHO_PGRP,
                    p,
                    class,
                    level,
                    args.ignore,
                    "pgid",
                );
            }
        }
        if let Some(ref uids) = args.uid {
            for &u in uids {
                failed |= set_ioprio(
                    IOPRIO_WHO_USER,
                    u,
                    class,
                    level,
                    args.ignore,
                    "uid",
                );
            }
        }
        return if failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    if set_ioprio(IOPRIO_WHO_PROCESS, 0, class, level, args.ignore, "pid")
        && !args.ignore
    {
        return ExitCode::FAILURE;
    }

    let err = exec_command(&args.command);
    eprintln!("ionice: failed to execute {}: {err}", args.command[0]);
    ExitCode::FAILURE
}

fn exec_command(command: &[String]) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    std::process::Command::new(&command[0])
        .args(&command[1..])
        .exec()
}
