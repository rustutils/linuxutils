use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs::{self, File},
    io::{self, BufRead},
    process::ExitCode,
};

/// Show information on System V IPC facilities.
///
/// Reads /proc/sysvipc/{shm,msg,sem} to display active IPC resources,
/// system limits, or usage summaries.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "ipcs", about = "Show information on System V IPC facilities")]
pub struct Args {
    /// Show shared memory segments
    #[arg(short = 'm', long)]
    shmems: bool,

    /// Show message queues
    #[arg(short = 'q', long)]
    queues: bool,

    /// Show semaphore arrays
    #[arg(short = 's', long)]
    semaphores: bool,

    /// Show all three resource types
    #[arg(short = 'a', long)]
    all: bool,

    /// Show system resource limits
    #[arg(short = 'l', long)]
    limits: bool,

    /// Show usage summary
    #[arg(short = 'u', long)]
    summary: bool,

    /// Show sizes in bytes
    #[arg(short = 'b', long)]
    bytes: bool,

    /// Show last-attach / last-detach / last-change times
    #[arg(short = 't', long)]
    time: bool,

    /// Show creator and last-operation PIDs
    #[arg(short = 'p', long)]
    pid: bool,

    /// Show creator and owner UIDs/GIDs
    #[arg(short = 'c', long)]
    creator: bool,
}

fn read_proc_val(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn show_shm_limits() {
    let shmmax = read_proc_val("/proc/sys/kernel/shmmax");
    let shmall = read_proc_val("/proc/sys/kernel/shmall");
    let shmmni = read_proc_val("/proc/sys/kernel/shmmni");

    println!();
    println!("------ Shared Memory Limits --------");
    println!("max number of segments = {shmmni}");
    println!(
        "max seg size (kbytes) = {}",
        shmmax.parse::<u64>().unwrap_or(0) / 1024
    );
    // `shmall` is in pages; the kernel reports a near-`u64::MAX`
    // value when there's no cap, so use saturating math to avoid an
    // overflow panic.
    let shmall_pages = shmall.parse::<u64>().unwrap_or(0);
    println!(
        "max total shared memory (kbytes) = {}",
        shmall_pages.saturating_mul(4096) / 1024
    );
    println!("min seg size (bytes) = 1");
}

fn show_msg_limits() {
    let msgmni = read_proc_val("/proc/sys/kernel/msgmni");
    let msgmax = read_proc_val("/proc/sys/kernel/msgmax");
    let msgmnb = read_proc_val("/proc/sys/kernel/msgmnb");

    println!();
    println!("------ Messages Limits --------");
    println!("max queues system wide = {msgmni}");
    println!("max size of message (bytes) = {msgmax}");
    println!("default max size of queue (bytes) = {msgmnb}");
}

fn show_sem_limits() {
    let sem = read_proc_val("/proc/sys/kernel/sem");
    let parts: Vec<&str> = sem.split_whitespace().collect();
    let semmsl = parts.first().unwrap_or(&"0");
    let semmns = parts.get(1).unwrap_or(&"0");
    let semopm = parts.get(2).unwrap_or(&"0");
    let semmni = parts.get(3).unwrap_or(&"0");

    println!();
    println!("------ Semaphore Limits --------");
    println!("max number of arrays = {semmni}");
    println!("max semaphores per array = {semmsl}");
    println!("max semaphores system wide = {semmns}");
    println!("max ops per semop call = {semopm}");
    println!("semaphore max value = 32767");
}

fn show_shm_summary() {
    let count = count_proc_entries("/proc/sysvipc/shm");
    println!();
    println!("------ Shared Memory Status --------");
    println!("segments allocated {count}");
}

fn show_msg_summary() {
    let count = count_proc_entries("/proc/sysvipc/msg");
    println!();
    println!("------ Messages Status --------");
    println!("allocated queues = {count}");
}

fn show_sem_summary() {
    let count = count_proc_entries("/proc/sysvipc/sem");
    println!();
    println!("------ Semaphore Status --------");
    println!("used arrays = {count}");
}

fn count_proc_entries(path: &str) -> usize {
    let Ok(file) = File::open(path) else {
        return 0;
    };
    io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .count()
}

fn show_shm_list() {
    println!();
    println!("------ Shared Memory Segments --------");
    // Header string is matched verbatim by the upstream regression
    // tests; column widths are tuned to reproduce it.
    println!(
        "key        shmid      owner      perms      bytes      nattch     status      "
    );

    let Ok(file) = File::open("/proc/sysvipc/shm") else {
        return;
    };
    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 14 {
            let key = format!("0x{:08x}", f[0].parse::<i64>().unwrap_or(0));
            let uid: u32 = f[7].parse().unwrap_or(0);
            let owner = uid_to_name(uid);
            println!(
                "{:<10} {:<10} {:<10} {:<10} {:>10} {:>6}     ",
                key, f[1], owner, f[2], f[3], f[5]
            );
        }
    }
}

fn show_msg_list() {
    println!();
    println!("------ Message Queues --------");
    println!(
        "key        msqid      owner      perms      used-bytes   messages    "
    );

    let Ok(file) = File::open("/proc/sysvipc/msg") else {
        return;
    };
    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 13 {
            let key = format!("0x{:08x}", f[0].parse::<i64>().unwrap_or(0));
            let uid: u32 = f[7].parse().unwrap_or(0);
            let owner = uid_to_name(uid);
            println!(
                "{:<10} {:<10} {:<10} {:<10} {:>10} {:>8}",
                key, f[1], owner, f[2], f[3], f[4]
            );
        }
    }
}

fn show_sem_list() {
    println!();
    println!("------ Semaphore Arrays --------");
    println!("key        semid      owner      perms      nsems     ");

    let Ok(file) = File::open("/proc/sysvipc/sem") else {
        return;
    };
    for line in io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 9 {
            let key = format!("0x{:08x}", f[0].parse::<i64>().unwrap_or(0));
            let uid: u32 = f[4].parse().unwrap_or(0);
            let owner = uid_to_name(uid);
            println!(
                "{:<10} {:<10} {:<10} {:<10} {:>5}",
                key, f[1], owner, f[2], f[3]
            );
        }
    }
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

pub fn run(args: Args) -> ExitCode {
    let none_specified = !args.shmems && !args.queues && !args.semaphores;
    let show_shm = args.shmems || args.all || none_specified;
    let show_msg = args.queues || args.all || none_specified;
    let show_sem = args.semaphores || args.all || none_specified;

    // Upstream prints sections in msg → shm → sem order regardless
    // of which mode is active.
    if args.limits {
        if show_msg {
            show_msg_limits();
        }
        if show_shm {
            show_shm_limits();
        }
        if show_sem {
            show_sem_limits();
        }
    } else if args.summary {
        if show_msg {
            show_msg_summary();
        }
        if show_shm {
            show_shm_summary();
        }
        if show_sem {
            show_sem_summary();
        }
    } else if args.time {
        if show_msg {
            show_msg_times();
        }
        if show_shm {
            show_shm_times();
        }
        if show_sem {
            show_sem_times();
        }
    } else if args.pid {
        if show_msg {
            show_msg_pids();
        }
        if show_shm {
            show_shm_pids();
        }
        // Upstream's ipcs prints no PID block for semaphores —
        // /proc/sysvipc/sem doesn't expose a last-op PID.
    } else if args.creator {
        if show_msg {
            show_msg_creators();
        }
        if show_shm {
            show_shm_creators();
        }
        if show_sem {
            show_sem_creators();
        }
    } else {
        if show_msg {
            show_msg_list();
        }
        if show_shm {
            show_shm_list();
        }
        if show_sem {
            show_sem_list();
        }
    }

    ExitCode::SUCCESS
}

// ---- --time mode -----------------------------------------------------------

fn show_shm_times() {
    println!();
    println!("------ Shared Memory Attach/Detach/Change Times --------");
    println!(
        "{:<10} {:<10} {:<20} {:<20} {:<20}",
        "shmid", "owner", "attached", "detached", "changed"
    );
    let _ = print_shm_rows(|f| {
        let owner = uid_to_name(f.uid);
        println!(
            "{:<10} {:<10} {:<20} {:<20} {:<20}",
            f.shmid,
            owner,
            time_or_dash(f.atime),
            time_or_dash(f.dtime),
            time_or_dash(f.ctime),
        );
    });
}

fn show_msg_times() {
    println!();
    println!("------ Message Queues Send/Recv/Change Times --------");
    println!(
        "{:<8} {:<10} {:<20} {:<20} {:<20}",
        "msqid", "owner", "send", "recv", "change"
    );
    let _ = print_msg_rows(|f| {
        let owner = uid_to_name(f.uid);
        println!(
            "{:<8} {:<10} {:<20} {:<20} {:<20}",
            f.msqid,
            owner,
            time_or_dash(f.stime),
            time_or_dash(f.rtime),
            time_or_dash(f.ctime),
        );
    });
}

fn show_sem_times() {
    println!();
    println!("------ Semaphore Operation/Change Times --------");
    println!(
        "{:<8} {:<10} {:<26} {:<26}",
        "semid", "owner", "last-op", "last-changed"
    );
    let _ = print_sem_rows(|f| {
        let owner = uid_to_name(f.uid);
        println!(
            "{:<8} {:<10} {:<26} {:<26}",
            f.semid,
            owner,
            time_or_dash(f.otime),
            time_or_dash(f.ctime),
        );
    });
}

// ---- --pid mode ------------------------------------------------------------

fn show_shm_pids() {
    println!();
    println!("------ Shared Memory Creator/Last-op PIDs --------");
    println!(
        "{:<10} {:<10} {:<10} {:<10}",
        "shmid", "owner", "cpid", "lpid"
    );
    let _ = print_shm_rows(|f| {
        let owner = uid_to_name(f.uid);
        println!(
            "{:<10} {:<10} {:<10} {:<10}",
            f.shmid, owner, f.cpid, f.lpid,
        );
    });
}

fn show_msg_pids() {
    println!();
    println!("------ Message Queues PIDs --------");
    println!(
        "{:<10} {:<10} {:<10} {:<10}",
        "msqid", "owner", "lspid", "lrpid"
    );
    let _ = print_msg_rows(|f| {
        let owner = uid_to_name(f.uid);
        println!(
            "{:<10} {:<10} {:<10} {:<10}",
            f.msqid, owner, f.lspid, f.lrpid,
        );
    });
}

// ---- --creator mode --------------------------------------------------------

fn show_shm_creators() {
    println!();
    println!("------ Shared Memory Segment Creators/Owners --------");
    println!(
        "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10}",
        "shmid", "perms", "cuid", "cgid", "uid", "gid"
    );
    let _ = print_shm_rows(|f| {
        println!(
            "{:<10} {:<10o} {:<10} {:<10} {:<10} {:<10}",
            f.shmid, f.perms, f.cuid, f.cgid, f.uid, f.gid,
        );
    });
}

fn show_msg_creators() {
    println!();
    println!("------ Message Queues Creators/Owners --------");
    println!(
        "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10}",
        "msqid", "perms", "cuid", "cgid", "uid", "gid"
    );
    let _ = print_msg_rows(|f| {
        println!(
            "{:<10} {:<10o} {:<10} {:<10} {:<10} {:<10}",
            f.msqid, f.perms, f.cuid, f.cgid, f.uid, f.gid,
        );
    });
}

fn show_sem_creators() {
    println!();
    println!("------ Semaphore Arrays Creators/Owners --------");
    println!(
        "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10}",
        "semid", "perms", "cuid", "cgid", "uid", "gid"
    );
    let _ = print_sem_rows(|f| {
        println!(
            "{:<10} {:<10o} {:<10} {:<10} {:<10} {:<10}",
            f.semid, f.perms, f.cuid, f.cgid, f.uid, f.gid,
        );
    });
}

// ---- /proc/sysvipc parsers -------------------------------------------------

struct ShmEntry {
    shmid: u64,
    perms: u32,
    cpid: u64,
    lpid: u64,
    uid: u32,
    gid: u32,
    cuid: u32,
    cgid: u32,
    atime: i64,
    dtime: i64,
    ctime: i64,
}

struct MsgEntry {
    msqid: u64,
    perms: u32,
    lspid: u64,
    lrpid: u64,
    uid: u32,
    gid: u32,
    cuid: u32,
    cgid: u32,
    stime: i64,
    rtime: i64,
    ctime: i64,
}

struct SemEntry {
    semid: u64,
    perms: u32,
    uid: u32,
    gid: u32,
    cuid: u32,
    cgid: u32,
    otime: i64,
    ctime: i64,
}

/// `/proc/sysvipc/shm` columns: key shmid perms size cpid lpid nattch
/// uid gid cuid cgid atime dtime ctime rss swap
fn print_shm_rows<F: FnMut(&ShmEntry)>(mut emit: F) -> std::io::Result<()> {
    let content = fs::read_to_string("/proc/sysvipc/shm").unwrap_or_default();
    for line in content.lines().skip(1) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() < 14 {
            continue;
        }
        emit(&ShmEntry {
            shmid: p[1].parse().unwrap_or(0),
            perms: u32::from_str_radix(p[2], 8).unwrap_or(0),
            cpid: p[4].parse().unwrap_or(0),
            lpid: p[5].parse().unwrap_or(0),
            uid: p[7].parse().unwrap_or(0),
            gid: p[8].parse().unwrap_or(0),
            cuid: p[9].parse().unwrap_or(0),
            cgid: p[10].parse().unwrap_or(0),
            atime: p[11].parse().unwrap_or(0),
            dtime: p[12].parse().unwrap_or(0),
            ctime: p[13].parse().unwrap_or(0),
        });
    }
    Ok(())
}

/// `/proc/sysvipc/msg` columns: key msqid perms cbytes qnum lspid
/// lrpid uid gid cuid cgid stime rtime ctime
fn print_msg_rows<F: FnMut(&MsgEntry)>(mut emit: F) -> std::io::Result<()> {
    let content = fs::read_to_string("/proc/sysvipc/msg").unwrap_or_default();
    for line in content.lines().skip(1) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() < 14 {
            continue;
        }
        emit(&MsgEntry {
            msqid: p[1].parse().unwrap_or(0),
            perms: u32::from_str_radix(p[2], 8).unwrap_or(0),
            lspid: p[5].parse().unwrap_or(0),
            lrpid: p[6].parse().unwrap_or(0),
            uid: p[7].parse().unwrap_or(0),
            gid: p[8].parse().unwrap_or(0),
            cuid: p[9].parse().unwrap_or(0),
            cgid: p[10].parse().unwrap_or(0),
            stime: p[11].parse().unwrap_or(0),
            rtime: p[12].parse().unwrap_or(0),
            ctime: p[13].parse().unwrap_or(0),
        });
    }
    Ok(())
}

/// `/proc/sysvipc/sem` columns: key semid perms nsems uid gid cuid
/// cgid otime ctime
fn print_sem_rows<F: FnMut(&SemEntry)>(mut emit: F) -> std::io::Result<()> {
    let content = fs::read_to_string("/proc/sysvipc/sem").unwrap_or_default();
    for line in content.lines().skip(1) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() < 10 {
            continue;
        }
        emit(&SemEntry {
            semid: p[1].parse().unwrap_or(0),
            perms: u32::from_str_radix(p[2], 8).unwrap_or(0),
            uid: p[4].parse().unwrap_or(0),
            gid: p[5].parse().unwrap_or(0),
            cuid: p[6].parse().unwrap_or(0),
            cgid: p[7].parse().unwrap_or(0),
            otime: p[8].parse().unwrap_or(0),
            ctime: p[9].parse().unwrap_or(0),
        });
    }
    Ok(())
}

/// Render a Unix epoch seconds value the way upstream does in
/// these column layouts: `Mon May 04 12:34:56 2026`, or `Not set`
/// if the timestamp is zero.
fn time_or_dash(t: i64) -> String {
    if t == 0 {
        return "Not set".to_string();
    }
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let tt = t as libc::time_t;
    unsafe { libc::localtime_r(&tt, &mut tm) };
    let mut buf = [0u8; 64];
    let len = unsafe {
        libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            c"%a %b %d %H:%M:%S %Y".as_ptr(),
            &tm,
        )
    };
    String::from_utf8_lossy(&buf[..len]).into_owned()
}
