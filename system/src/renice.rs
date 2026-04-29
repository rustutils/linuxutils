use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::process::{
    Pid, Uid, getpriority_pgrp, getpriority_process, getpriority_user,
    setpriority_pgrp, setpriority_process, setpriority_user,
};
use std::{ffi::CString, process::ExitCode};

#[derive(Parser)]
#[command(
    name = "renice",
    about = "Alter priority of running processes",
    override_usage = "renice [-n|--priority|--relative] priority [-p|--pid] pid...\n       \
                      renice [-n|--priority|--relative] priority -g|--pgrp pgid...\n       \
                      renice [-n|--priority|--relative] priority -u|--user user..."
)]
pub struct Args {
    /// Specify the scheduling priority (absolute by default)
    #[arg(short = 'n', long = "priority", allow_hyphen_values = true)]
    priority_flag: Option<i32>,

    /// Use relative priority adjustment instead of absolute
    #[arg(long = "relative")]
    relative: bool,

    /// Interpret targets as process IDs (default)
    #[arg(short = 'p', long = "pid")]
    pid: bool,

    /// Interpret targets as process group IDs
    #[arg(short = 'g', long = "pgrp")]
    pgrp: bool,

    /// Interpret targets as usernames or UIDs
    #[arg(short = 'u', long = "user")]
    user: bool,

    /// Priority value and targets
    #[arg(required = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn resolve_uid(name: &str) -> Option<u32> {
    let c_name = CString::new(name).ok()?;
    let pw = unsafe { libc::getpwnam(c_name.as_ptr()) };
    if pw.is_null() {
        None
    } else {
        Some(unsafe { (*pw).pw_uid })
    }
}

fn renice_pid(
    pid_val: u32,
    priority: i32,
    relative: bool,
) -> Result<(i32, i32), String> {
    let pid = Pid::from_raw(pid_val as i32)
        .ok_or_else(|| "invalid pid 0".to_string())?;
    let old = getpriority_process(Some(pid)).map_err(|e| format!("{e}"))?;
    let new_prio = if relative { old + priority } else { priority };
    setpriority_process(Some(pid), new_prio).map_err(|e| format!("{e}"))?;
    let actual = getpriority_process(Some(pid)).map_err(|e| format!("{e}"))?;
    Ok((old, actual))
}

fn renice_pgrp(
    pgid_val: u32,
    priority: i32,
    relative: bool,
) -> Result<(i32, i32), String> {
    let pgid = if pgid_val == 0 {
        None
    } else {
        Some(
            Pid::from_raw(pgid_val as i32)
                .ok_or_else(|| "invalid pgid".to_string())?,
        )
    };
    let old = getpriority_pgrp(pgid).map_err(|e| format!("{e}"))?;
    let new_prio = if relative { old + priority } else { priority };
    setpriority_pgrp(pgid, new_prio).map_err(|e| format!("{e}"))?;
    let actual = getpriority_pgrp(pgid).map_err(|e| format!("{e}"))?;
    Ok((old, actual))
}

fn renice_user(
    uid_val: u32,
    priority: i32,
    relative: bool,
) -> Result<(i32, i32), String> {
    let uid = Uid::from_raw(uid_val);
    let old = getpriority_user(uid).map_err(|e| format!("{e}"))?;
    let new_prio = if relative { old + priority } else { priority };
    setpriority_user(uid, new_prio).map_err(|e| format!("{e}"))?;
    let actual = getpriority_user(uid).map_err(|e| format!("{e}"))?;
    Ok((old, actual))
}

pub fn run(args: Args) -> ExitCode {
    let (priority, targets) = if let Some(p) = args.priority_flag {
        (p, args.args.as_slice())
    } else {
        let Some(first) = args.args.first() else {
            eprintln!("renice: missing priority");
            return ExitCode::FAILURE;
        };
        let p: i32 = match first.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("renice: invalid priority '{first}'");
                return ExitCode::FAILURE;
            }
        };
        if args.args.len() < 2 {
            eprintln!("renice: missing target");
            return ExitCode::FAILURE;
        }
        (p, &args.args[1..])
    };

    let mut failed = false;

    for target in targets {
        if args.user {
            let uid = match target.parse::<u32>() {
                Ok(uid) => uid,
                Err(_) => match resolve_uid(target) {
                    Some(uid) => uid,
                    None => {
                        eprintln!("renice: unknown user {target}");
                        failed = true;
                        continue;
                    }
                },
            };
            match renice_user(uid, priority, args.relative) {
                Ok((old, new)) => println!(
                    "{uid} (user ID) old priority {old}, new priority {new}"
                ),
                Err(e) => {
                    eprintln!(
                        "renice: failed to set priority for user {target}: {e}"
                    );
                    failed = true;
                }
            }
        } else if args.pgrp {
            let pgid: u32 = match target.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("renice: invalid process group ID '{target}'");
                    failed = true;
                    continue;
                }
            };
            match renice_pgrp(pgid, priority, args.relative) {
                Ok((old, new)) => {
                    println!(
                        "{pgid} (process group ID) old priority {old}, new priority {new}"
                    )
                }
                Err(e) => {
                    eprintln!(
                        "renice: failed to set priority for pgid {pgid}: {e}"
                    );
                    failed = true;
                }
            }
        } else {
            let pid: u32 = match target.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("renice: invalid process ID '{target}'");
                    failed = true;
                    continue;
                }
            };
            match renice_pid(pid, priority, args.relative) {
                Ok((old, new)) => {
                    println!(
                        "{pid} (process ID) old priority {old}, new priority {new}"
                    )
                }
                Err(e) => {
                    eprintln!(
                        "renice: failed to set priority for pid {pid}: {e}"
                    );
                    failed = true;
                }
            }
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
