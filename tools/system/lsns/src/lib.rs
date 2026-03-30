use clap::Parser;
use cols::{OutputMode, Table, WidthHint, print_table};
use std::{collections::HashMap, fs, process::ExitCode};

const NS_TYPES: &[&str] =
    &["mnt", "net", "ipc", "user", "pid", "uts", "cgroup", "time"];

#[derive(Parser)]
#[command(about = "List system namespaces")]
pub struct Args {
    /// Use JSON output format
    #[arg(short = 'J', long)]
    json: bool,

    /// Use list format output
    #[arg(short, long)]
    list: bool,

    /// Don't print headings
    #[arg(short = 'n', long)]
    noheadings: bool,

    /// Output columns to print
    #[arg(short, long, value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Output all columns
    #[arg(long)]
    output_all: bool,

    /// Use raw output format
    #[arg(short, long)]
    raw: bool,

    /// Namespace type filter
    #[arg(short = 't', long = "type")]
    ns_type: Option<String>,

    /// Print namespaces for a specific PID
    #[arg(short = 'p', long = "task")]
    task: Option<u32>,

    /// Optional namespace inode to show
    #[arg()]
    namespace: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Col {
    Ns,
    Type,
    Nprocs,
    Pid,
    Ppid,
    Command,
    Uid,
    User,
}

impl Col {
    fn name(self) -> &'static str {
        match self {
            Col::Ns => "NS",
            Col::Type => "TYPE",
            Col::Nprocs => "NPROCS",
            Col::Pid => "PID",
            Col::Ppid => "PPID",
            Col::Command => "COMMAND",
            Col::Uid => "UID",
            Col::User => "USER",
        }
    }

    fn whint(self) -> WidthHint {
        match self {
            Col::Ns => WidthHint::Fixed(10),
            Col::Type => WidthHint::Fixed(6),
            Col::Nprocs => WidthHint::Fixed(6),
            Col::Pid => WidthHint::Fixed(6),
            Col::Ppid => WidthHint::Fixed(6),
            Col::Command => WidthHint::Auto,
            Col::Uid => WidthHint::Fixed(6),
            Col::User => WidthHint::Fixed(8),
        }
    }

    fn is_right(self) -> bool {
        matches!(
            self,
            Col::Ns | Col::Nprocs | Col::Pid | Col::Ppid | Col::Uid
        )
    }

    fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "NS" => Some(Col::Ns),
            "TYPE" => Some(Col::Type),
            "NPROCS" => Some(Col::Nprocs),
            "PID" => Some(Col::Pid),
            "PPID" => Some(Col::Ppid),
            "COMMAND" => Some(Col::Command),
            "UID" => Some(Col::Uid),
            "USER" => Some(Col::User),
            _ => None,
        }
    }
}

const DEFAULT_COLUMNS: &[Col] = &[
    Col::Ns,
    Col::Type,
    Col::Nprocs,
    Col::Pid,
    Col::User,
    Col::Command,
];

const ALL_COLUMNS: &[Col] = &[
    Col::Ns,
    Col::Type,
    Col::Nprocs,
    Col::Pid,
    Col::Ppid,
    Col::Command,
    Col::Uid,
    Col::User,
];

#[derive(Debug)]
struct NsInfo {
    ino: u64,
    ns_type: String,
    nprocs: u32,
    pid: u32,
    ppid: u32,
    uid: u32,
    user: String,
    command: String,
}

fn read_proc_pid(pid: u32) -> Option<(u32, u32, String)> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Parse: "pid (comm) state ppid ..."
    // comm can contain spaces and parens, so find the last ')'.
    let comm_end = stat.rfind(')')?;
    let after_comm = &stat[comm_end + 2..]; // skip ") "
    let fields: Vec<&str> = after_comm.split_whitespace().collect();
    // fields[0] = state, fields[1] = ppid
    let ppid: u32 = fields.get(1)?.parse().ok()?;

    let cmdline = fs::read_to_string(format!("/proc/{pid}/cmdline"))
        .ok()
        .map(|s| s.replace('\0', " ").trim().to_string())
        .unwrap_or_default();

    let command = if cmdline.is_empty() {
        // Kernel thread — use comm.
        let comm_start = stat.find('(')? + 1;
        format!("[{}]", &stat[comm_start..comm_end])
    } else {
        cmdline
    };

    Some((ppid, 0, command))
}

fn get_uid(pid: u32) -> u32 {
    fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()
        .and_then(|s| {
            for line in s.lines() {
                if let Some(rest) = line.strip_prefix("Uid:") {
                    return rest.split_whitespace().next()?.parse().ok();
                }
            }
            None
        })
        .unwrap_or(0)
}

fn uid_to_name(uid: u32) -> String {
    fs::read_to_string("/etc/passwd")
        .ok()
        .and_then(|content| {
            for line in content.lines() {
                let fields: Vec<&str> = line.split(':').collect();
                if fields.len() >= 3
                    && let Ok(u) = fields[2].parse::<u32>()
                    && u == uid
                {
                    return Some(fields[0].to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| uid.to_string())
}

fn read_ns_inode(pid: u32, ns_type: &str) -> Option<u64> {
    let link = fs::read_link(format!("/proc/{pid}/ns/{ns_type}")).ok()?;
    // Format: "type:[inode]"
    let s = link.to_str()?;
    let start = s.find('[')? + 1;
    let end = s.find(']')?;
    s[start..end].parse().ok()
}

fn scan_namespaces(
    type_filter: Option<&str>,
    task_filter: Option<u32>,
    ns_filter: Option<u64>,
) -> Vec<NsInfo> {
    // Map: (ns_type, inode) -> NsInfo
    let mut ns_map: HashMap<(String, u64), NsInfo> = HashMap::new();

    let pids: Vec<u32> = if let Some(pid) = task_filter {
        vec![pid]
    } else {
        fs::read_dir("/proc")
            .ok()
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|e| e.file_name().to_str()?.parse::<u32>().ok())
                    .collect()
            })
            .unwrap_or_default()
    };

    let types: Vec<&str> = if let Some(t) = type_filter {
        NS_TYPES.iter().copied().filter(|&ns| ns == t).collect()
    } else {
        NS_TYPES.to_vec()
    };

    for &pid in &pids {
        for &ns_type in &types {
            let ino = match read_ns_inode(pid, ns_type) {
                Some(i) => i,
                None => continue,
            };

            if let Some(filter) = ns_filter
                && ino != filter
            {
                continue;
            }

            let key = (ns_type.to_string(), ino);
            let entry = ns_map.entry(key).or_insert_with(|| {
                let uid = get_uid(pid);
                let (ppid, _, command) =
                    read_proc_pid(pid).unwrap_or((0, 0, String::new()));
                NsInfo {
                    ino,
                    ns_type: ns_type.to_string(),
                    nprocs: 0,
                    pid,
                    ppid,
                    uid,
                    user: uid_to_name(uid),
                    command,
                }
            });
            entry.nprocs += 1;

            // Track the lowest PID.
            if pid < entry.pid {
                let uid = get_uid(pid);
                let (ppid, _, command) =
                    read_proc_pid(pid).unwrap_or((0, 0, String::new()));
                entry.pid = pid;
                entry.ppid = ppid;
                entry.uid = uid;
                entry.user = uid_to_name(uid);
                entry.command = command;
            }
        }
    }

    let mut result: Vec<NsInfo> = ns_map.into_values().collect();
    result.sort_by_key(|ns| ns.ino);
    result
}

pub fn run(args: Args) -> ExitCode {
    let columns = if args.output_all {
        ALL_COLUMNS.to_vec()
    } else if let Some(ref names) = args.output {
        let mut cols = Vec::new();
        for name in names {
            match Col::from_name(name.trim()) {
                Some(c) => cols.push(c),
                None => {
                    eprintln!("lsns: unknown column: {name}");
                    return ExitCode::FAILURE;
                }
            }
        }
        cols
    } else {
        DEFAULT_COLUMNS.to_vec()
    };

    let type_filter = args.ns_type.as_deref();
    let namespaces = scan_namespaces(type_filter, args.task, args.namespace);

    let mut table = Table::new();
    table.set_name("namespaces");

    if args.json {
        table.set_output_mode(OutputMode::Json);
    } else if args.raw {
        table.set_output_mode(OutputMode::Raw);
    }

    if args.noheadings {
        table.enable_no_headings(true);
    }

    for col in &columns {
        let idx = table.new_column(col.name());
        table.column_mut(idx).unwrap().set_width_hint(col.whint());
        if col.is_right() {
            table.column_mut(idx).unwrap().set_right(true);
        }
    }

    for ns in &namespaces {
        let line_id = table.new_line(None);
        let line = table.line_mut(line_id);

        for (ci, col) in columns.iter().enumerate() {
            let val = match col {
                Col::Ns => ns.ino.to_string(),
                Col::Type => ns.ns_type.clone(),
                Col::Nprocs => ns.nprocs.to_string(),
                Col::Pid => ns.pid.to_string(),
                Col::Ppid => ns.ppid.to_string(),
                Col::Command => ns.command.clone(),
                Col::Uid => ns.uid.to_string(),
                Col::User => ns.user.clone(),
            };
            line.set_data(ci, &val);
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = print_table(&table, &mut out) {
        eprintln!("lsns: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
