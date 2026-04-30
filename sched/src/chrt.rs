use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::process::ExitCode;

const SCHED_OTHER: i32 = 0;
const SCHED_FIFO: i32 = 1;
const SCHED_RR: i32 = 2;
const SCHED_BATCH: i32 = 3;
const SCHED_IDLE: i32 = 5;
const SCHED_DEADLINE: i32 = 6;
const SCHED_RESET_ON_FORK: i32 = 0x40000000;

#[repr(C)]
#[derive(Default)]
struct SchedAttr {
    size: u32,
    sched_policy: u32,
    sched_flags: u64,
    sched_nice: i32,
    sched_priority: u32,
    sched_runtime: u64,
    sched_deadline: u64,
    sched_period: u64,
}

fn policy_name(policy: i32) -> &'static str {
    match policy {
        SCHED_OTHER => "SCHED_OTHER",
        SCHED_FIFO => "SCHED_FIFO",
        SCHED_RR => "SCHED_RR",
        SCHED_BATCH => "SCHED_BATCH",
        SCHED_IDLE => "SCHED_IDLE",
        SCHED_DEADLINE => "SCHED_DEADLINE",
        _ => "unknown",
    }
}

fn get_scheduler(pid: i32) -> Result<i32, std::io::Error> {
    let ret = unsafe { libc::sched_getscheduler(pid) };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

fn get_priority(pid: i32) -> Result<i32, std::io::Error> {
    let mut param: libc::sched_param = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::sched_getparam(pid, &mut param) };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(param.sched_priority)
    }
}

fn set_scheduler(
    pid: i32,
    policy: i32,
    priority: i32,
) -> Result<(), std::io::Error> {
    let param = libc::sched_param {
        sched_priority: priority,
    };
    let ret = unsafe { libc::sched_setscheduler(pid, policy, &param) };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn get_priority_min(policy: i32) -> i32 {
    unsafe { libc::sched_get_priority_min(policy) }
}

fn get_priority_max(policy: i32) -> i32 {
    unsafe { libc::sched_get_priority_max(policy) }
}

fn sched_getattr(pid: i32) -> Result<SchedAttr, std::io::Error> {
    let mut attr = SchedAttr {
        size: std::mem::size_of::<SchedAttr>() as u32,
        ..Default::default()
    };
    let ret = unsafe {
        libc::syscall(
            libc::SYS_sched_getattr,
            pid as libc::c_uint,
            &mut attr as *mut SchedAttr,
            attr.size,
            0u32,
        )
    };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(attr)
    }
}

fn sched_setattr(pid: i32, attr: &SchedAttr) -> Result<(), std::io::Error> {
    let ret = unsafe {
        libc::syscall(
            libc::SYS_sched_setattr,
            pid as libc::c_uint,
            attr as *const SchedAttr,
            0u32,
        )
    };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn print_sched_info(pid: i32) -> Result<(), std::io::Error> {
    let raw_policy = get_scheduler(pid)?;
    let reset_on_fork = raw_policy & SCHED_RESET_ON_FORK != 0;
    let policy = raw_policy & !SCHED_RESET_ON_FORK;
    let priority = get_priority(pid)?;

    let name = policy_name(policy);
    let reset_str = if reset_on_fork {
        "|SCHED_RESET_ON_FORK"
    } else {
        ""
    };
    println!("pid {pid}'s current scheduling policy: {name}{reset_str}");
    println!("pid {pid}'s current scheduling priority: {priority}");

    if policy == SCHED_DEADLINE
        && let Ok(attr) = sched_getattr(pid)
    {
        println!(
            "pid {pid}'s current runtime/deadline/period parameters: {}/{}/{}",
            attr.sched_runtime, attr.sched_deadline, attr.sched_period
        );
    }

    Ok(())
}

fn show_max_priorities() {
    let policies = [
        SCHED_OTHER,
        SCHED_FIFO,
        SCHED_RR,
        SCHED_BATCH,
        SCHED_IDLE,
        SCHED_DEADLINE,
    ];
    for &p in &policies {
        let min = get_priority_min(p);
        let max = get_priority_max(p);
        println!("{} min/max priority\t: {}/{}", policy_name(p), min, max);
    }
}

fn get_task_pids(pid: i32) -> Result<Vec<i32>, std::io::Error> {
    let task_dir = format!("/proc/{pid}/task");
    let mut pids = Vec::new();
    for entry in std::fs::read_dir(&task_dir)? {
        let entry = entry?;
        if let Ok(tid) = entry.file_name().to_string_lossy().parse::<i32>() {
            pids.push(tid);
        }
    }
    Ok(pids)
}

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "chrt",
    about = "Manipulate the real-time attributes of a process",
    override_usage = "chrt [options] <priority> <command> [<arg>...]\n       \
                      chrt [options] --pid [-p] <priority> <pid>\n       \
                      chrt [options] --pid [-p] <pid>"
)]
pub struct Args {
    /// Operate on an existing PID
    #[arg(short = 'p', long = "pid")]
    pid: bool,

    /// Show min/max valid priorities, then exit
    #[arg(short = 'm', long = "max")]
    max: bool,

    /// Show status information
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Set SCHED_RESET_ON_FORK flag
    #[arg(short = 'R', long = "reset-on-fork")]
    reset_on_fork: bool,

    /// Operate on all tasks (threads) of a given PID
    #[arg(short = 'a', long = "all-tasks")]
    all_tasks: bool,

    /// Set scheduling policy to SCHED_OTHER
    #[arg(short = 'o', long = "other")]
    other: bool,

    /// Set scheduling policy to SCHED_FIFO
    #[arg(short = 'f', long = "fifo")]
    fifo: bool,

    /// Set scheduling policy to SCHED_RR
    #[arg(short = 'r', long = "rr")]
    rr: bool,

    /// Set scheduling policy to SCHED_BATCH
    #[arg(short = 'b', long = "batch")]
    batch: bool,

    /// Set scheduling policy to SCHED_IDLE
    #[arg(short = 'i', long = "idle")]
    idle: bool,

    /// Set scheduling policy to SCHED_DEADLINE
    #[arg(short = 'd', long = "deadline")]
    deadline: bool,

    /// Runtime parameter for SCHED_DEADLINE (nanoseconds)
    #[arg(short = 'T', long = "sched-runtime")]
    sched_runtime: Option<u64>,

    /// Period parameter for SCHED_DEADLINE (nanoseconds)
    #[arg(short = 'P', long = "sched-period")]
    sched_period: Option<u64>,

    /// Deadline parameter for SCHED_DEADLINE (nanoseconds)
    #[arg(short = 'D', long = "sched-deadline")]
    sched_deadline: Option<u64>,

    /// Positional arguments: priority and/or PID/command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

impl Args {
    fn selected_policy(&self) -> Option<i32> {
        if self.other {
            Some(SCHED_OTHER)
        } else if self.fifo {
            Some(SCHED_FIFO)
        } else if self.rr {
            Some(SCHED_RR)
        } else if self.batch {
            Some(SCHED_BATCH)
        } else if self.idle {
            Some(SCHED_IDLE)
        } else if self.deadline {
            Some(SCHED_DEADLINE)
        } else {
            None
        }
    }
}

pub fn run(args: Args) -> ExitCode {
    if args.max {
        show_max_priorities();
        return ExitCode::SUCCESS;
    }

    if args.pid {
        return run_pid_mode(&args);
    }

    run_command_mode(&args)
}

fn run_pid_mode(args: &Args) -> ExitCode {
    let positional = &args.args;

    if positional.is_empty() {
        eprintln!("chrt: missing PID argument");
        return ExitCode::FAILURE;
    }

    let policy_selected = args.selected_policy().is_some();

    // Determine if this is query or set mode:
    // - If a policy flag is set, we're in set mode: first arg is priority, second is PID
    // - If no policy flag and only one positional arg, it's query mode (arg is PID)
    // - If no policy flag and two positional args, it's set mode (priority PID) with default RR
    if !policy_selected && positional.len() == 1 {
        // Query mode: chrt -p PID
        let pid = match positional[0].parse::<i32>() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("chrt: invalid PID: '{}'", positional[0]);
                return ExitCode::FAILURE;
            }
        };

        if args.all_tasks {
            match get_task_pids(pid) {
                Ok(tids) => {
                    for tid in tids {
                        if let Err(e) = print_sched_info(tid) {
                            eprintln!(
                                "chrt: failed to get pid {tid}'s policy: {e}"
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("chrt: failed to read tasks for pid {pid}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        } else if let Err(e) = print_sched_info(pid) {
            eprintln!("chrt: failed to get pid {pid}'s policy: {e}");
            return ExitCode::FAILURE;
        }

        return ExitCode::SUCCESS;
    }

    // Set mode: chrt [policy] -p priority PID
    if positional.len() < 2 {
        eprintln!("chrt: missing priority or PID argument");
        return ExitCode::FAILURE;
    }

    let priority = match positional[0].parse::<i32>() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("chrt: invalid priority: '{}'", positional[0]);
            return ExitCode::FAILURE;
        }
    };

    let pid = match positional[1].parse::<i32>() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("chrt: invalid PID: '{}'", positional[1]);
            return ExitCode::FAILURE;
        }
    };

    let policy = args.selected_policy().unwrap_or(SCHED_RR);
    let full_policy = if args.reset_on_fork {
        policy | SCHED_RESET_ON_FORK
    } else {
        policy
    };

    let pids = if args.all_tasks {
        match get_task_pids(pid) {
            Ok(tids) => tids,
            Err(e) => {
                eprintln!("chrt: failed to read tasks for pid {pid}: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        vec![pid]
    };

    for &target_pid in &pids {
        if args.verbose {
            let _ = print_sched_info(target_pid);
        }

        if policy == SCHED_DEADLINE {
            if let Err(code) = set_deadline(args, target_pid, full_policy) {
                return code;
            }
        } else if let Err(e) = set_scheduler(target_pid, full_policy, priority)
        {
            eprintln!("chrt: failed to set pid {target_pid}'s policy: {e}");
            return ExitCode::FAILURE;
        }

        if args.verbose {
            let _ = print_sched_info(target_pid);
        }
    }

    ExitCode::SUCCESS
}

fn run_command_mode(args: &Args) -> ExitCode {
    let positional = &args.args;

    if positional.is_empty() {
        eprintln!("chrt: missing priority argument");
        return ExitCode::FAILURE;
    }

    let priority = match positional[0].parse::<i32>() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("chrt: invalid priority: '{}'", positional[0]);
            return ExitCode::FAILURE;
        }
    };

    if positional.len() < 2 {
        eprintln!("chrt: missing command argument");
        return ExitCode::FAILURE;
    }

    let command = &positional[1..];
    let policy = args.selected_policy().unwrap_or(SCHED_RR);
    let full_policy = if args.reset_on_fork {
        policy | SCHED_RESET_ON_FORK
    } else {
        policy
    };

    if args.verbose {
        let _ = print_sched_info(0);
    }

    if policy == SCHED_DEADLINE {
        if let Err(code) = set_deadline(args, 0, full_policy) {
            return code;
        }
    } else if let Err(e) = set_scheduler(0, full_policy, priority) {
        eprintln!("chrt: failed to set scheduler: {e}");
        return ExitCode::FAILURE;
    }

    if args.verbose {
        let _ = print_sched_info(0);
    }

    let err = exec_command(command);
    eprintln!("chrt: failed to execute {}: {err}", command[0]);
    ExitCode::FAILURE
}

fn set_deadline(
    args: &Args,
    pid: i32,
    full_policy: i32,
) -> Result<(), ExitCode> {
    let period = match args.sched_period {
        Some(p) => p,
        None => {
            eprintln!("chrt: --sched-period is required for SCHED_DEADLINE");
            return Err(ExitCode::FAILURE);
        }
    };

    let deadline = args.sched_deadline.unwrap_or(period);
    let runtime = args.sched_runtime.unwrap_or(deadline);

    let attr = SchedAttr {
        size: std::mem::size_of::<SchedAttr>() as u32,
        sched_policy: (full_policy & !SCHED_RESET_ON_FORK) as u32,
        sched_flags: if full_policy & SCHED_RESET_ON_FORK != 0 {
            0x01
        } else {
            0
        },
        sched_nice: 0,
        sched_priority: 0,
        sched_runtime: runtime,
        sched_deadline: deadline,
        sched_period: period,
    };

    if let Err(e) = sched_setattr(pid, &attr) {
        eprintln!("chrt: failed to set pid {pid}'s policy: {e}");
        return Err(ExitCode::FAILURE);
    }

    Ok(())
}

fn exec_command(command: &[String]) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    std::process::Command::new(&command[0])
        .args(&command[1..])
        .exec()
}
