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
    println!(
        "max total shared memory (kbytes) = {}",
        shmall.parse::<u64>().unwrap_or(0) * 4096 / 1024
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
    println!(
        "{:<10} {:<10} {:<10} {:<10} {:>10} {:>6}     status",
        "key", "shmid", "owner", "perms", "bytes", "nattch"
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
        "{:<10} {:<10} {:<10} {:<10} {:>10} {:>8}",
        "key", "msqid", "owner", "perms", "used-bytes", "messages"
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
    println!(
        "{:<10} {:<10} {:<10} {:<10} {:>5}",
        "key", "semid", "owner", "perms", "nsems"
    );

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

    if args.limits {
        if show_shm {
            show_shm_limits();
        }
        if show_msg {
            show_msg_limits();
        }
        if show_sem {
            show_sem_limits();
        }
    } else if args.summary {
        if show_shm {
            show_shm_summary();
        }
        if show_msg {
            show_msg_summary();
        }
        if show_sem {
            show_sem_summary();
        }
    } else {
        if show_shm {
            show_shm_list();
        }
        if show_msg {
            show_msg_list();
        }
        if show_sem {
            show_sem_list();
        }
    }

    ExitCode::SUCCESS
}
