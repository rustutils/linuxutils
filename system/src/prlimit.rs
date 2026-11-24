use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use cols::{OutputMode, Table, WidthHint, print_table};
use rustix::{
    fs::fstat,
    process::{
        Pid, PidfdFlags, Resource, Rlimit, getrlimit, pidfd_open, prlimit,
        setrlimit,
    },
};
use std::{fs, os::unix::process::CommandExt, process, process::ExitCode};

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "prlimit",
    about = "Get and set process resource limits",
    override_usage = "prlimit [options] [--<resource>=<limit>] [-p PID]\n       \
                      prlimit [options] [--<resource>=<limit>] COMMAND [args...]"
)]
pub struct Args {
    /// Process ID. Can be a plain PID, or the race-free PID:INODE form
    /// printed by getino -p. With an inode, it is checked before any limit
    /// is changed, so a reused PID can't be changed by accident.
    #[arg(short, long, value_name = "pid")]
    pid: Option<String>,

    /// Define which output columns to use
    #[arg(short = 'o', long, value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Don't print headings
    #[arg(long)]
    noheadings: bool,

    /// Use the raw output format
    #[arg(long)]
    raw: bool,

    // Resource limits: absent means not selected, a bare flag means
    // show only, and flag=limit means set and show.
    /// Max size of core files (bytes)
    #[arg(short = 'c', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    core: Option<String>,

    /// Max size of a process's data segment (bytes)
    #[arg(short = 'd', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    data: Option<String>,

    /// Max nice priority allowed to raise
    #[arg(short = 'e', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    nice: Option<String>,

    /// Max size of files written by the process (bytes)
    #[arg(short = 'f', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    fsize: Option<String>,

    /// Max number of pending signals
    #[arg(short = 'i', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    sigpending: Option<String>,

    /// Max locked-in-memory address space (bytes)
    #[arg(short = 'l', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    memlock: Option<String>,

    /// Max resident set size (bytes)
    #[arg(short = 'm', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    rss: Option<String>,

    /// Max number of open files
    #[arg(short = 'n', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    nofile: Option<String>,

    /// Max bytes in POSIX message queues (bytes)
    #[arg(short = 'q', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    msgqueue: Option<String>,

    /// Max real-time scheduling priority
    #[arg(short = 'r', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    rtprio: Option<String>,

    /// Max stack size (bytes)
    #[arg(short = 's', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    stack: Option<String>,

    /// Max CPU time (seconds)
    #[arg(short = 't', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    cpu: Option<String>,

    /// Max number of user processes
    #[arg(short = 'u', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    nproc: Option<String>,

    /// Max virtual memory size (bytes)
    #[arg(short = 'v', long = "as", value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    addr_space: Option<String>,

    /// Max number of file locks held
    #[arg(short = 'x', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    locks: Option<String>,

    /// CPU time for real-time tasks (microseconds)
    #[arg(short = 'y', long, value_name = "limit", require_equals = true, num_args = 0..=1, default_missing_value = "")]
    rttime: Option<String>,

    /// Command and arguments to run with new limits applied
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ResourceInfo {
    resource: Resource,
    name: &'static str,
    description: &'static str,
    units: &'static str,
}

const ALL_RESOURCES: &[ResourceInfo] = &[
    ResourceInfo {
        resource: Resource::As,
        name: "AS",
        description: "address space limit",
        units: "bytes",
    },
    ResourceInfo {
        resource: Resource::Core,
        name: "CORE",
        description: "max core file size",
        units: "bytes",
    },
    ResourceInfo {
        resource: Resource::Cpu,
        name: "CPU",
        description: "CPU time",
        units: "seconds",
    },
    ResourceInfo {
        resource: Resource::Data,
        name: "DATA",
        description: "max data size",
        units: "bytes",
    },
    ResourceInfo {
        resource: Resource::Fsize,
        name: "FSIZE",
        description: "max file size",
        units: "bytes",
    },
    ResourceInfo {
        resource: Resource::Locks,
        name: "LOCKS",
        description: "max number of file locks held",
        units: "locks",
    },
    ResourceInfo {
        resource: Resource::Memlock,
        name: "MEMLOCK",
        description: "max locked-in-memory address space",
        units: "bytes",
    },
    ResourceInfo {
        resource: Resource::Msgqueue,
        name: "MSGQUEUE",
        description: "max bytes in POSIX mqueues",
        units: "bytes",
    },
    ResourceInfo {
        resource: Resource::Nice,
        name: "NICE",
        description: "max nice prio allowed to raise",
        units: "",
    },
    ResourceInfo {
        resource: Resource::Nofile,
        name: "NOFILE",
        description: "max number of open files",
        units: "files",
    },
    ResourceInfo {
        resource: Resource::Nproc,
        name: "NPROC",
        description: "max number of processes",
        units: "processes",
    },
    ResourceInfo {
        resource: Resource::Rss,
        name: "RSS",
        description: "max resident set size",
        units: "bytes",
    },
    ResourceInfo {
        resource: Resource::Rtprio,
        name: "RTPRIO",
        description: "max real-time priority",
        units: "",
    },
    ResourceInfo {
        resource: Resource::Rttime,
        name: "RTTIME",
        description: "timeout for real-time tasks",
        units: "microsecs",
    },
    ResourceInfo {
        resource: Resource::Sigpending,
        name: "SIGPENDING",
        description: "max number of pending signals",
        units: "signals",
    },
    ResourceInfo {
        resource: Resource::Stack,
        name: "STACK",
        description: "max stack size",
        units: "bytes",
    },
];

fn resource_spec(args: &Args) -> Vec<(ResourceInfo, Option<&str>)> {
    // Build a list of (resource, optional_limit_str) for selected resources.
    // A None limit means query only, and Some(s) means set then query.
    macro_rules! entry {
        ($field:expr, $name:expr) => {
            if let Some(ref val) = $field {
                let ri = ALL_RESOURCES
                    .iter()
                    .find(|r| r.name == $name)
                    .copied()
                    .unwrap();
                let limit = if val.is_empty() {
                    None
                } else {
                    Some(val.as_str())
                };
                Some((ri, limit))
            } else {
                None
            }
        };
    }

    let selected: Vec<Option<(ResourceInfo, Option<&str>)>> = vec![
        entry!(args.addr_space, "AS"),
        entry!(args.core, "CORE"),
        entry!(args.cpu, "CPU"),
        entry!(args.data, "DATA"),
        entry!(args.fsize, "FSIZE"),
        entry!(args.locks, "LOCKS"),
        entry!(args.memlock, "MEMLOCK"),
        entry!(args.msgqueue, "MSGQUEUE"),
        entry!(args.nice, "NICE"),
        entry!(args.nofile, "NOFILE"),
        entry!(args.nproc, "NPROC"),
        entry!(args.rss, "RSS"),
        entry!(args.rtprio, "RTPRIO"),
        entry!(args.rttime, "RTTIME"),
        entry!(args.sigpending, "SIGPENDING"),
        entry!(args.stack, "STACK"),
    ];

    selected.into_iter().flatten().collect()
}

fn parse_limit(s: &str) -> Result<(Option<u64>, Option<u64>), String> {
    // Formats: "soft:hard", "soft:", ":hard", "value"
    if let Some(colon) = s.find(':') {
        let soft_str = &s[..colon];
        let hard_str = &s[colon + 1..];
        let soft = parse_one_limit(soft_str)?;
        let hard = parse_one_limit(hard_str)?;
        Ok((soft, hard))
    } else {
        let v = parse_single_limit(s)?;
        Ok((Some(v), Some(v)))
    }
}

// Returns None for empty (= don't change this half)
fn parse_one_limit(s: &str) -> Result<Option<u64>, String> {
    if s.is_empty() {
        Ok(None)
    } else {
        parse_single_limit(s).map(Some)
    }
}

fn parse_single_limit(s: &str) -> Result<u64, String> {
    match s {
        "unlimited" | "infinity" => Ok(u64::MAX),
        _ => s.parse::<u64>().map_err(|_| format!("invalid limit: {s}")),
    }
}

fn limit_str(val: Option<u64>) -> String {
    match val {
        None | Some(u64::MAX) => "unlimited".to_string(),
        Some(v) => v.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Col {
    Resource,
    Description,
    Soft,
    Hard,
    Units,
}

impl Col {
    fn name(self) -> &'static str {
        match self {
            Col::Resource => "RESOURCE",
            Col::Description => "DESCRIPTION",
            Col::Soft => "SOFT",
            Col::Hard => "HARD",
            Col::Units => "UNITS",
        }
    }

    fn whint(self) -> WidthHint {
        match self {
            Col::Resource => WidthHint::Fixed(10),
            Col::Description => WidthHint::Fixed(36),
            Col::Soft => WidthHint::Fixed(9),
            Col::Hard => WidthHint::Fixed(9),
            Col::Units => WidthHint::Fixed(9),
        }
    }

    fn is_right(self) -> bool {
        matches!(self, Col::Soft | Col::Hard)
    }

    fn from_name(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "RESOURCE" => Some(Col::Resource),
            "DESCRIPTION" => Some(Col::Description),
            "SOFT" => Some(Col::Soft),
            "HARD" => Some(Col::Hard),
            "UNITS" => Some(Col::Units),
            _ => None,
        }
    }
}

const DEFAULT_COLUMNS: &[Col] = &[
    Col::Resource,
    Col::Description,
    Col::Soft,
    Col::Hard,
    Col::Units,
];

// Read a resource limit for the given PID (or self if None).
// Uses getrlimit for self, /proc/<pid>/limits for other PIDs.
fn read_limit(pid: Option<u32>, resource: Resource) -> Option<Rlimit> {
    if let Some(pid) = pid {
        read_limit_from_proc(pid, resource)
    } else {
        Some(getrlimit(resource))
    }
}

// Parse /proc/<pid>/limits to get the limit for a specific resource.
fn read_limit_from_proc(pid: u32, resource: Resource) -> Option<Rlimit> {
    let content = fs::read_to_string(format!("/proc/{pid}/limits")).ok()?;
    // Map resource to the proc "Limit" prefix string.
    let proc_name = match resource {
        Resource::Cpu => "Max cpu time",
        Resource::Fsize => "Max file size",
        Resource::Data => "Max data size",
        Resource::Stack => "Max stack size",
        Resource::Core => "Max core file size",
        Resource::Rss => "Max resident set",
        Resource::Nproc => "Max processes",
        Resource::Nofile => "Max open files",
        Resource::Memlock => "Max locked memory",
        Resource::As => "Max address space",
        Resource::Locks => "Max file locks",
        Resource::Sigpending => "Max pending signals",
        Resource::Msgqueue => "Max msgqueue size",
        Resource::Nice => "Max nice priority",
        Resource::Rtprio => "Max realtime priority",
        Resource::Rttime => "Max realtime timeout",
        _ => return None,
    };
    for line in content.lines().skip(1) {
        if line.starts_with(proc_name) {
            let rest = line.strip_prefix(proc_name).unwrap().trim();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let current = parse_proc_limit(parts[0]);
                let maximum = parse_proc_limit(parts[1]);
                return Some(Rlimit { current, maximum });
            }
        }
    }
    None
}

fn parse_proc_limit(s: &str) -> Option<u64> {
    if s == "unlimited" {
        None
    } else {
        s.parse().ok()
    }
}

fn u64_to_rlimit_half(v: u64) -> Option<u64> {
    if v == u64::MAX { None } else { Some(v) }
}

/// Parse a `--pid` value: either bare `PID` or `PID:INODE`. The
/// inode (if present) is checked against the pidfs inode of the
/// resolved pidfd to detect PID reuse before we touch limits.
fn resolve_pid(arg: &str) -> Result<u32, String> {
    let (pid_str, expected_inode) = match arg.split_once(':') {
        Some((p, i)) => {
            let inode = i
                .parse::<u64>()
                .map_err(|_| format!("invalid PID:INODE inode: {arg}"))?;
            (p, Some(inode))
        }
        None => (arg, None),
    };
    let pid: u32 =
        pid_str.parse().map_err(|_| format!("invalid PID: {arg}"))?;
    if pid == 0 {
        return Err(format!("invalid PID: {arg}"));
    }

    if let Some(expected) = expected_inode {
        let raw = Pid::from_raw(pid as i32)
            .ok_or_else(|| format!("invalid PID: {arg}"))?;
        let fd = pidfd_open(raw, PidfdFlags::empty())
            .map_err(|e| format!("pidfd_open({pid}): {e}"))?;
        let st = fstat(&fd).map_err(|e| format!("fstat: {e}"))?;
        if st.st_ino != expected {
            return Err(format!("PID {pid} has been recycled"));
        }
    }
    Ok(pid)
}

pub fn run(args: Args) -> ExitCode {
    let selected = resource_spec(&args);

    // Determine which resources to display.
    let display: Vec<ResourceInfo> = if selected.is_empty() {
        ALL_RESOURCES.to_vec()
    } else {
        selected.iter().map(|(ri, _)| *ri).collect()
    };

    let target_pid: Option<u32> = match args.pid.as_deref() {
        Some(s) => match resolve_pid(s) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("prlimit: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => None,
    };

    // Apply any limit changes.
    for (ri, limit_str_opt) in &selected {
        let Some(limit_str_val) = limit_str_opt else {
            continue;
        };
        let (new_soft, new_hard) = match parse_limit(limit_str_val) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("prlimit: {e}");
                return ExitCode::FAILURE;
            }
        };

        // Read current limits to fill in unchanged halves.
        let current = match read_limit(target_pid, ri.resource) {
            Some(r) => r,
            None => {
                eprintln!("prlimit: {}: failed to read current limit", ri.name);
                return ExitCode::FAILURE;
            }
        };

        let merged = Rlimit {
            current: new_soft
                .map(u64_to_rlimit_half)
                .unwrap_or(current.current),
            maximum: new_hard
                .map(u64_to_rlimit_half)
                .unwrap_or(current.maximum),
        };

        let result = if let Some(pid) = target_pid {
            let raw_pid = unsafe { Pid::from_raw_unchecked(pid as i32) };
            prlimit(Some(raw_pid), ri.resource, merged).map(|_| ())
        } else {
            setrlimit(ri.resource, merged)
        };

        if let Err(e) = result {
            eprintln!("prlimit: failed to set {}: {e}", ri.name);
            return ExitCode::FAILURE;
        }
    }

    // If a command was given, exec it now (limits already applied above).
    if !args.command.is_empty() {
        let (prog, prog_args) = args.command.split_first().unwrap();
        let err = process::Command::new(prog).args(prog_args).exec();
        eprintln!("prlimit: {prog}: {err}");
        return ExitCode::FAILURE;
    }

    // Match upstream `prlimit(1)`: when *any* limit is being set
    // (i.e. there's a `--<res>=…` value, not just a bare `--<res>`),
    // suppress the table. The user asked us to set, not show.
    let any_setting = selected.iter().any(|(_, v)| v.is_some());
    if any_setting {
        return ExitCode::SUCCESS;
    }

    // Display limits.
    let columns: Vec<Col> = if let Some(ref names) = args.output {
        let mut cols = Vec::new();
        for name in names {
            match Col::from_name(name.trim()) {
                Some(c) => cols.push(c),
                None => {
                    eprintln!("prlimit: unknown column: {name}");
                    return ExitCode::FAILURE;
                }
            }
        }
        cols
    } else {
        DEFAULT_COLUMNS.to_vec()
    };

    let mut table = Table::new();
    if args.raw {
        table.output_mode_set(OutputMode::Raw);
    }
    if args.noheadings {
        table.headings_set(false);
    }
    for col in &columns {
        let idx = table.new_column(col.name());
        table.column_mut(idx).unwrap().width_hint_set(col.whint());
        if col.is_right() {
            table.column_mut(idx).unwrap().right_set(true);
        }
    }

    for ri in &display {
        let lim = match read_limit(target_pid, ri.resource) {
            Some(r) => r,
            None => {
                eprintln!("prlimit: {}: failed to read limit", ri.name);
                return ExitCode::FAILURE;
            }
        };

        let line_id = table.new_line(None);
        let line = table.line_mut(line_id);

        for (ci, col) in columns.iter().enumerate() {
            let val = match col {
                Col::Resource => ri.name.to_string(),
                Col::Description => ri.description.to_string(),
                Col::Soft => limit_str(lim.current),
                Col::Hard => limit_str(lim.maximum),
                Col::Units => ri.units.to_string(),
            };
            line.data_set(ci, &val);
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = print_table(&table, &mut out) {
        eprintln!("prlimit: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
