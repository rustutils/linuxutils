use clap::Parser;
use cols::{Cols, print_table};
use std::{
    fs::{self, File},
    io::{self, BufRead},
    process::ExitCode,
};

/// List information on IPC facilities.
///
/// Modern replacement for ipcs. Reads /proc/sysvipc/* and
/// /proc/sys/kernel/* to display IPC resource information
/// in tabular format.
#[derive(Parser)]
#[command(name = "lsipc", about = "List information on IPC facilities")]
pub struct Args {
    /// List shared memory segments
    #[arg(short = 'm', long)]
    shmems: bool,

    /// List message queues
    #[arg(short = 'q', long)]
    queues: bool,

    /// List semaphore arrays
    #[arg(short = 's', long)]
    semaphores: bool,

    /// Show system-wide usage and limits
    #[arg(short = 'g', long)]
    global: bool,

    /// Suppress table headers
    #[arg(long)]
    noheadings: bool,

    /// Show sizes in bytes
    #[arg(short = 'b', long)]
    bytes: bool,
}

fn read_val(path: &str) -> u64 {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(0)
}

fn count_entries(path: &str) -> u64 {
    let Ok(file) = File::open(path) else {
        return 0;
    };
    io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .count() as u64
}

#[derive(Cols)]
struct GlobalRow {
    #[column(header = "RESOURCE")]
    resource: String,

    #[column(header = "DESCRIPTION")]
    description: String,

    #[column(right, header = "LIMIT")]
    limit: String,

    #[column(right, header = "USED")]
    used: String,

    #[column(right, header = "USE%")]
    use_pct: String,
}

#[derive(Cols)]
struct ShmRow {
    #[column(right, header = "KEY")]
    key: String,

    #[column(right, header = "ID")]
    id: String,

    #[column(header = "OWNER")]
    owner: String,

    #[column(right, header = "PERMS")]
    perms: String,

    #[column(right, header = "SIZE")]
    size: String,

    #[column(right, header = "NATTCH")]
    nattch: String,
}

#[derive(Cols)]
struct MsgRow {
    #[column(right, header = "KEY")]
    key: String,

    #[column(right, header = "ID")]
    id: String,

    #[column(header = "OWNER")]
    owner: String,

    #[column(right, header = "PERMS")]
    perms: String,

    #[column(right, header = "USED-BYTES")]
    used_bytes: String,

    #[column(right, header = "MESSAGES")]
    messages: String,
}

#[derive(Cols)]
struct SemRow {
    #[column(right, header = "KEY")]
    key: String,

    #[column(right, header = "ID")]
    id: String,

    #[column(header = "OWNER")]
    owner: String,

    #[column(right, header = "PERMS")]
    perms: String,

    #[column(right, header = "NSEMS")]
    nsems: String,
}

fn uid_to_name(uid: u32) -> String {
    fs::read_to_string("/etc/passwd")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 && parts[2].parse::<u32>().ok() == Some(uid)
                {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| uid.to_string())
}

fn pct(used: u64, limit: u64) -> String {
    if limit == 0 {
        "0.00%".to_string()
    } else {
        format!("{:.2}%", used as f64 / limit as f64 * 100.0)
    }
}

fn show_global(noheadings: bool) {
    let msgmni = read_val("/proc/sys/kernel/msgmni");
    let msgmnb = read_val("/proc/sys/kernel/msgmnb");
    let msgmax = read_val("/proc/sys/kernel/msgmax");
    let shmmni = read_val("/proc/sys/kernel/shmmni");
    let shmall = read_val("/proc/sys/kernel/shmall");
    let shmmax = read_val("/proc/sys/kernel/shmmax");

    let sem_str =
        fs::read_to_string("/proc/sys/kernel/sem").unwrap_or_default();
    let sem_parts: Vec<u64> = sem_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    let semmsl = sem_parts.first().copied().unwrap_or(0);
    let semmns = sem_parts.get(1).copied().unwrap_or(0);
    let semopm = sem_parts.get(2).copied().unwrap_or(0);
    let semmni = sem_parts.get(3).copied().unwrap_or(0);

    let msg_used = count_entries("/proc/sysvipc/msg");
    let shm_used = count_entries("/proc/sysvipc/shm");
    let sem_used = count_entries("/proc/sysvipc/sem");

    let rows = vec![
        GlobalRow {
            resource: "MSGMNI".into(),
            description: "Number of message queues".into(),
            limit: msgmni.to_string(),
            used: msg_used.to_string(),
            use_pct: pct(msg_used, msgmni),
        },
        GlobalRow {
            resource: "MSGMAX".into(),
            description: "Max size of message (bytes)".into(),
            limit: msgmax.to_string(),
            used: "-".into(),
            use_pct: "-".into(),
        },
        GlobalRow {
            resource: "MSGMNB".into(),
            description: "Default max size of queue (bytes)".into(),
            limit: msgmnb.to_string(),
            used: "-".into(),
            use_pct: "-".into(),
        },
        GlobalRow {
            resource: "SHMMNI".into(),
            description: "Shared memory segments".into(),
            limit: shmmni.to_string(),
            used: shm_used.to_string(),
            use_pct: pct(shm_used, shmmni),
        },
        GlobalRow {
            resource: "SHMALL".into(),
            description: "Shared memory pages".into(),
            limit: shmall.to_string(),
            used: "-".into(),
            use_pct: "-".into(),
        },
        GlobalRow {
            resource: "SHMMAX".into(),
            description: "Max segment size (bytes)".into(),
            limit: shmmax.to_string(),
            used: "-".into(),
            use_pct: "-".into(),
        },
        GlobalRow {
            resource: "SEMMNI".into(),
            description: "Number of semaphore identifiers".into(),
            limit: semmni.to_string(),
            used: sem_used.to_string(),
            use_pct: pct(sem_used, semmni),
        },
        GlobalRow {
            resource: "SEMMNS".into(),
            description: "Total number of semaphores".into(),
            limit: semmns.to_string(),
            used: "-".into(),
            use_pct: "-".into(),
        },
        GlobalRow {
            resource: "SEMMSL".into(),
            description: "Max semaphores per array".into(),
            limit: semmsl.to_string(),
            used: "-".into(),
            use_pct: "-".into(),
        },
        GlobalRow {
            resource: "SEMOPM".into(),
            description: "Max ops per semop call".into(),
            limit: semopm.to_string(),
            used: "-".into(),
            use_pct: "-".into(),
        },
    ];

    let mut table = GlobalRow::to_table(&rows);
    table.headings_set(!noheadings);
    let _ = print_table(&table, &mut io::stdout().lock());
}

fn show_shm(noheadings: bool) {
    let Ok(file) = File::open("/proc/sysvipc/shm") else {
        return;
    };
    let mut rows = Vec::new();
    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 14 {
            let uid: u32 = f[7].parse().unwrap_or(0);
            rows.push(ShmRow {
                key: format!("0x{:08x}", f[0].parse::<i64>().unwrap_or(0)),
                id: f[1].to_string(),
                owner: uid_to_name(uid),
                perms: f[2].to_string(),
                size: f[3].to_string(),
                nattch: f[5].to_string(),
            });
        }
    }
    let mut table = ShmRow::to_table(&rows);
    table.headings_set(!noheadings);
    let _ = print_table(&table, &mut io::stdout().lock());
}

fn show_msg(noheadings: bool) {
    let Ok(file) = File::open("/proc/sysvipc/msg") else {
        return;
    };
    let mut rows = Vec::new();
    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 13 {
            let uid: u32 = f[7].parse().unwrap_or(0);
            rows.push(MsgRow {
                key: format!("0x{:08x}", f[0].parse::<i64>().unwrap_or(0)),
                id: f[1].to_string(),
                owner: uid_to_name(uid),
                perms: f[2].to_string(),
                used_bytes: f[3].to_string(),
                messages: f[4].to_string(),
            });
        }
    }
    let mut table = MsgRow::to_table(&rows);
    table.headings_set(!noheadings);
    let _ = print_table(&table, &mut io::stdout().lock());
}

fn show_sem(noheadings: bool) {
    let Ok(file) = File::open("/proc/sysvipc/sem") else {
        return;
    };
    let mut rows = Vec::new();
    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 9 {
            let uid: u32 = f[4].parse().unwrap_or(0);
            rows.push(SemRow {
                key: format!("0x{:08x}", f[0].parse::<i64>().unwrap_or(0)),
                id: f[1].to_string(),
                owner: uid_to_name(uid),
                perms: f[2].to_string(),
                nsems: f[3].to_string(),
            });
        }
    }
    let mut table = SemRow::to_table(&rows);
    table.headings_set(!noheadings);
    let _ = print_table(&table, &mut io::stdout().lock());
}

pub fn run(args: Args) -> ExitCode {
    let specific = args.shmems || args.queues || args.semaphores;

    if args.global || !specific {
        if !specific {
            show_global(args.noheadings);
            return ExitCode::SUCCESS;
        }
        show_global(args.noheadings);
        return ExitCode::SUCCESS;
    }

    if args.shmems {
        show_shm(args.noheadings);
    }
    if args.queues {
        show_msg(args.noheadings);
    }
    if args.semaphores {
        show_sem(args.noheadings);
    }

    ExitCode::SUCCESS
}
