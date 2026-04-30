use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead},
    process::ExitCode,
};

/// Remove System V IPC resources.
///
/// Uses shmctl(2), msgctl(2), or semctl(2) with IPC_RMID to remove
/// shared memory segments, message queues, or semaphore arrays.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "ipcrm", about = "Remove System V IPC resources")]
pub struct Args {
    /// Remove shared memory segment by ID
    #[arg(short = 'm', long = "shmem-id", action = clap::ArgAction::Append)]
    shmem_id: Vec<String>,

    /// Remove shared memory segment by key
    #[arg(short = 'M', long = "shmem-key", action = clap::ArgAction::Append)]
    shmem_key: Vec<String>,

    /// Remove message queue by ID
    #[arg(short = 'q', long = "queue-id", action = clap::ArgAction::Append)]
    queue_id: Vec<String>,

    /// Remove message queue by key
    #[arg(short = 'Q', long = "queue-key", action = clap::ArgAction::Append)]
    queue_key: Vec<String>,

    /// Remove semaphore array by ID
    #[arg(short = 's', long = "semaphore-id", action = clap::ArgAction::Append)]
    semaphore_id: Vec<String>,

    /// Remove semaphore array by key
    #[arg(short = 'S', long = "semaphore-key", action = clap::ArgAction::Append)]
    semaphore_key: Vec<String>,

    /// Remove all IPC resources (optionally: shm, msg, sem)
    #[arg(short = 'a', long, num_args = 0..=1, default_missing_value = "all")]
    all: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

fn parse_id(s: &str) -> Result<i32, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        i32::from_str_radix(hex, 16).map_err(|_| format!("invalid id: {s}"))
    } else if s.starts_with('0') && s.len() > 1 {
        i32::from_str_radix(s, 8).map_err(|_| format!("invalid id: {s}"))
    } else {
        s.parse().map_err(|_| format!("invalid id: {s}"))
    }
}

fn rm_shm(id: i32) -> io::Result<()> {
    if unsafe { libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut()) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn rm_msg(id: i32) -> io::Result<()> {
    if unsafe { libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut()) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn rm_sem(id: i32) -> io::Result<()> {
    if unsafe { libc::semctl(id, 0, libc::IPC_RMID) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn key_to_shmid(key: i32) -> io::Result<i32> {
    let id = unsafe { libc::shmget(key, 0, 0) };
    if id < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(id)
    }
}

fn key_to_msgid(key: i32) -> io::Result<i32> {
    let id = unsafe { libc::msgget(key, 0) };
    if id < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(id)
    }
}

fn key_to_semid(key: i32) -> io::Result<i32> {
    let id = unsafe { libc::semget(key, 0, 0) };
    if id < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(id)
    }
}

fn read_ids_from_proc(path: &str) -> Vec<i32> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .skip(1)
        .filter_map(|line| {
            line.split_whitespace().nth(1).and_then(|s| s.parse().ok())
        })
        .collect()
}

fn try_remove(
    kind: &str,
    id: i32,
    f: fn(i32) -> io::Result<()>,
    verbose: bool,
) -> bool {
    if verbose {
        eprintln!("ipcrm: removing {kind} id {id}");
    }
    if let Err(e) = f(id) {
        eprintln!("ipcrm: {kind} id {id}: {e}");
        return true;
    }
    false
}

pub fn run(args: Args) -> ExitCode {
    let mut failed = false;

    if let Some(ref filter) = args.all {
        let do_shm = filter == "all" || filter == "shm";
        let do_msg = filter == "all" || filter == "msg";
        let do_sem = filter == "all" || filter == "sem";

        if do_shm {
            for id in read_ids_from_proc("/proc/sysvipc/shm") {
                failed |= try_remove("shm", id, rm_shm, args.verbose);
            }
        }
        if do_msg {
            for id in read_ids_from_proc("/proc/sysvipc/msg") {
                failed |= try_remove("msg", id, rm_msg, args.verbose);
            }
        }
        if do_sem {
            for id in read_ids_from_proc("/proc/sysvipc/sem") {
                failed |= try_remove("sem", id, rm_sem, args.verbose);
            }
        }
    }

    for s in &args.shmem_id {
        match parse_id(s) {
            Ok(id) => failed |= try_remove("shm", id, rm_shm, args.verbose),
            Err(e) => {
                eprintln!("ipcrm: {e}");
                failed = true;
            }
        }
    }
    for s in &args.shmem_key {
        match parse_id(s)
            .and_then(|k| key_to_shmid(k).map_err(|e| e.to_string()))
        {
            Ok(id) => failed |= try_remove("shm", id, rm_shm, args.verbose),
            Err(e) => {
                eprintln!("ipcrm: shm key {s}: {e}");
                failed = true;
            }
        }
    }
    for s in &args.queue_id {
        match parse_id(s) {
            Ok(id) => failed |= try_remove("msg", id, rm_msg, args.verbose),
            Err(e) => {
                eprintln!("ipcrm: {e}");
                failed = true;
            }
        }
    }
    for s in &args.queue_key {
        match parse_id(s)
            .and_then(|k| key_to_msgid(k).map_err(|e| e.to_string()))
        {
            Ok(id) => failed |= try_remove("msg", id, rm_msg, args.verbose),
            Err(e) => {
                eprintln!("ipcrm: msg key {s}: {e}");
                failed = true;
            }
        }
    }
    for s in &args.semaphore_id {
        match parse_id(s) {
            Ok(id) => failed |= try_remove("sem", id, rm_sem, args.verbose),
            Err(e) => {
                eprintln!("ipcrm: {e}");
                failed = true;
            }
        }
    }
    for s in &args.semaphore_key {
        match parse_id(s)
            .and_then(|k| key_to_semid(k).map_err(|e| e.to_string()))
        {
            Ok(id) => failed |= try_remove("sem", id, rm_sem, args.verbose),
            Err(e) => {
                eprintln!("ipcrm: sem key {s}: {e}");
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
