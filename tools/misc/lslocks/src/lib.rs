use clap::Parser;
use smartcols::{OutputMode, Table, print_table};
use std::{
    collections::HashMap, fs, os::unix::fs::MetadataExt, process::ExitCode,
};

#[derive(Parser)]
#[command(name = "lslocks", about = "List local system locks")]
pub struct Args {
    /// Print SIZE in bytes rather than human-readable format
    #[arg(short, long)]
    bytes: bool,

    /// Use JSON output format
    #[arg(short = 'J', long)]
    json: bool,

    /// Ignore locks without read permissions on the file
    #[arg(short = 'i', long)]
    noinaccessible: bool,

    /// Don't print headings
    #[arg(short = 'n', long)]
    noheadings: bool,

    /// Define which output columns to use
    #[arg(short = 'o', long, value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Output all available columns
    #[arg(long)]
    output_all: bool,

    /// Display only locks held by this PID
    #[arg(short = 'p', long, value_name = "pid")]
    pid: Option<u32>,

    /// Use the raw output format
    #[arg(short = 'r', long)]
    raw: bool,

    /// Don't truncate text in columns
    #[arg(short = 'u', long)]
    notruncate: bool,
}

#[derive(Debug)]
struct Lock {
    lock_type: String, // FLOCK, POSIX, OFDLCK, LEASE, ...
    mandatory: bool,
    mode: String, // READ, WRITE
    pid: u32,
    major: u64,
    minor: u64,
    inode: u64,
    start: u64,
    end: Option<u64>, // None = EOF
    blocker: Option<u32>,
    command: String,
    path: String,
}

impl Lock {
    fn file_size(&self) -> Option<u64> {
        // Don't show size for zero-range locks (start=0, end=0).
        if self.start == 0 && self.end == Some(0) {
            return None;
        }
        if self.path.is_empty() {
            return None;
        }
        fs::metadata(&self.path).ok().map(|m| m.len())
    }

    fn size_str(&self, human: bool) -> String {
        match self.file_size() {
            None => String::new(),
            Some(b) if human => human_size(b),
            Some(b) => b.to_string(),
        }
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut val = bytes as f64;
    let mut unit = 0;
    while val >= 1024.0 && unit + 1 < UNITS.len() {
        val /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes}B")
    } else {
        format!("{:.0}{}", val, UNITS[unit])
    }
}

// Parse /proc/locks. Returns a list of Lock entries.
// Format: "id: TYPE ADVISORY|MANDATORY ACCESS PID MAJOR:MINOR:INODE START END"
// Blocked locks are indicated by a leading space or by repeated lock id.
fn parse_proc_locks() -> Vec<Lock> {
    let content = match fs::read_to_string("/proc/locks") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut locks = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split_whitespace().collect();
        // Expected: id: TYPE ADVISORY PID DEVICE START END
        // or:       id: TYPE ADVISORY PID DEVICE START END (blocked=id)
        if fields.len() < 8 {
            continue;
        }

        // fields[0] = "N:" (lock id, possibly with leading space for blocked)
        // Check if this is a blocked lock (kernel marks them with " ->" style)
        let _lock_id = fields[0].trim_end_matches(':');

        let lock_type = fields[1].to_string();
        let mandatory = fields[2] == "MANDATORY";
        let mode = fields[3].to_string();

        let pid: u32 = match fields[4].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // fields[5] = "MAJOR:MINOR:INODE"
        let (major, minor, inode) = match parse_device_inode(fields[5]) {
            Some(v) => v,
            None => continue,
        };

        let start: u64 = fields[6].parse().unwrap_or(0);
        let end: Option<u64> = if fields[7] == "EOF" {
            None
        } else {
            fields[7].parse().ok()
        };

        locks.push(Lock {
            lock_type,
            mandatory,
            mode,
            pid,
            major,
            minor,
            inode,
            start,
            end,
            blocker: None,
            command: String::new(),
            path: String::new(),
        });
    }

    locks
}

fn parse_device_inode(s: &str) -> Option<(u64, u64, u64)> {
    // Format: "MAJOR:MINOR:INODE" where MAJOR and MINOR are hex, INODE is decimal.
    let mut parts = s.splitn(3, ':');
    let major = u64::from_str_radix(parts.next()?, 16).ok()?;
    let minor = u64::from_str_radix(parts.next()?, 16).ok()?;
    let inode: u64 = parts.next()?.parse().ok()?;
    Some((major, minor, inode))
}

// Build a map from (pid, inode) → path by scanning /proc/<pid>/fd/.
// We key by (pid, inode) rather than (dev, inode) because the block device
// reported in /proc/locks may differ from st_dev (e.g. btrfs subvolumes).
fn build_path_cache(pids: &[u32]) -> HashMap<(u32, u64), String> {
    let mut cache = HashMap::new();

    for &pid in pids {
        let fd_dir = format!("/proc/{pid}/fd");
        let entries = match fs::read_dir(&fd_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let link = match fs::read_link(entry.path()) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let path_str = link.to_string_lossy().into_owned();

            // Stat via the fd symlink to get the inode.
            if let Ok(meta) = fs::metadata(entry.path()) {
                let ino = meta.ino();
                cache.entry((pid, ino)).or_insert(path_str);
            }
        }
    }

    cache
}

fn read_command(pid: u32) -> String {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Col {
    Command,
    Pid,
    Type,
    Size,
    Mode,
    Mandatory,
    Start,
    End,
    Path,
    Inode,
    MajMin,
    Blocker,
    Holders,
}

impl Col {
    fn name(self) -> &'static str {
        match self {
            Col::Command => "COMMAND",
            Col::Pid => "PID",
            Col::Type => "TYPE",
            Col::Size => "SIZE",
            Col::Mode => "MODE",
            Col::Mandatory => "M",
            Col::Start => "START",
            Col::End => "END",
            Col::Path => "PATH",
            Col::Inode => "INODE",
            Col::MajMin => "MAJ:MIN",
            Col::Blocker => "BLOCKER",
            Col::Holders => "HOLDERS",
        }
    }

    fn whint(self) -> f64 {
        match self {
            Col::Command => 14.0,
            Col::Pid => 5.0,
            Col::Type => 5.0,
            Col::Size => 5.0,
            Col::Mode => 5.0,
            Col::Mandatory => 1.0,
            Col::Start => 10.0,
            Col::End => 10.0,
            Col::Path => 0.0,
            Col::Inode => 8.0,
            Col::MajMin => 6.0,
            Col::Blocker => 7.0,
            Col::Holders => 0.0,
        }
    }

    fn is_right(self) -> bool {
        matches!(
            self,
            Col::Pid | Col::Start | Col::End | Col::Inode | Col::Blocker
        )
    }

    fn from_name(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "COMMAND" => Some(Col::Command),
            "PID" => Some(Col::Pid),
            "TYPE" => Some(Col::Type),
            "SIZE" => Some(Col::Size),
            "MODE" => Some(Col::Mode),
            "M" => Some(Col::Mandatory),
            "START" => Some(Col::Start),
            "END" => Some(Col::End),
            "PATH" => Some(Col::Path),
            "INODE" => Some(Col::Inode),
            "MAJ:MIN" => Some(Col::MajMin),
            "BLOCKER" => Some(Col::Blocker),
            "HOLDERS" => Some(Col::Holders),
            _ => None,
        }
    }
}

const DEFAULT_COLUMNS: &[Col] = &[
    Col::Command,
    Col::Pid,
    Col::Type,
    Col::Size,
    Col::Mode,
    Col::Mandatory,
    Col::Start,
    Col::End,
    Col::Path,
];

const ALL_COLUMNS: &[Col] = &[
    Col::Command,
    Col::Pid,
    Col::Type,
    Col::Size,
    Col::Inode,
    Col::MajMin,
    Col::Mode,
    Col::Mandatory,
    Col::Start,
    Col::End,
    Col::Path,
    Col::Blocker,
    Col::Holders,
];

pub fn run(args: Args) -> ExitCode {
    let mut locks = parse_proc_locks();

    // Filter by PID if requested.
    if let Some(filter_pid) = args.pid {
        locks.retain(|l| l.pid == filter_pid);
    }

    // Collect unique PIDs for path + command resolution.
    let pids: Vec<u32> = {
        let mut v: Vec<u32> = locks.iter().map(|l| l.pid).collect();
        v.sort_unstable();
        v.dedup();
        v
    };

    // Resolve paths from /proc/<pid>/fd/.
    let path_cache = build_path_cache(&pids);

    // Fill in command and path for each lock.
    for lock in &mut locks {
        lock.command = read_command(lock.pid);
        if let Some(p) = path_cache.get(&(lock.pid, lock.inode)) {
            lock.path = p.clone();
        }
    }

    if args.noinaccessible {
        locks.retain(|l| !l.path.is_empty());
    }

    let columns: Vec<Col> = if args.output_all {
        ALL_COLUMNS.to_vec()
    } else if let Some(ref names) = args.output {
        let mut cols = Vec::new();
        for name in names {
            match Col::from_name(name.trim()) {
                Some(c) => cols.push(c),
                None => {
                    eprintln!("lslocks: unknown column: {name}");
                    return ExitCode::FAILURE;
                }
            }
        }
        cols
    } else {
        DEFAULT_COLUMNS.to_vec()
    };

    let mut table = Table::new();
    table.set_name("locks");

    if args.json {
        table.set_output_mode(OutputMode::Json);
    } else if args.raw {
        table.set_output_mode(OutputMode::Raw);
    }

    if args.noheadings {
        table.enable_no_headings(true);
    }

    for col in &columns {
        let idx = table.new_column(col.name(), col.whint());
        if col.is_right() {
            table.column_mut(idx).unwrap().set_right(true);
        }
    }

    for lock in &locks {
        let line_id = table.new_line(None);
        let line = table.line_mut(line_id);

        for (ci, col) in columns.iter().enumerate() {
            let val = match col {
                Col::Command => lock.command.clone(),
                Col::Pid => lock.pid.to_string(),
                Col::Type => lock.lock_type.clone(),
                Col::Size => lock.size_str(!args.bytes),
                Col::Mode => lock.mode.clone(),
                Col::Mandatory => (lock.mandatory as u8).to_string(),
                Col::Start => lock.start.to_string(),
                Col::End => {
                    lock.end.map_or("EOF".to_string(), |e| e.to_string())
                }
                Col::Path => lock.path.clone(),
                Col::Inode => lock.inode.to_string(),
                Col::MajMin => format!("{}:{}", lock.major, lock.minor),
                Col::Blocker => {
                    lock.blocker.map_or(String::new(), |b| b.to_string())
                }
                Col::Holders => String::new(), // not implemented
            };
            line.set_data(ci, &val);
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = print_table(&table, &mut out) {
        eprintln!("lslocks: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
