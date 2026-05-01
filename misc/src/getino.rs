use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::{
    fs::fstat,
    process::{Pid, PidfdFlags, pidfd_open},
};
use std::{os::unix::fs::MetadataExt, process::ExitCode};

/// Print the inode number of a process's pidfd or one of its
/// namespaces, given a PID.
///
/// The kernel guarantees that a pidfs inode is not reused during the
/// current boot, so the `PID:inode` form (`-p`) is a race-free handle
/// for that process — usable e.g. with `kill PID:inode`. Namespace
/// inodes match those reported under `/proc/<pid>/ns/<type>`.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "getino", about = "Print pidfd or namespace inode for a PID")]
pub struct Args {
    /// Print the pidfs file-descriptor inode (default)
    #[arg(long, group = "kind")]
    pidfs: bool,

    /// Print the cgroup namespace inode
    #[arg(long, group = "kind")]
    cgroupns: bool,

    /// Print the IPC namespace inode
    #[arg(long, group = "kind")]
    ipcns: bool,

    /// Print the network namespace inode
    #[arg(long, group = "kind")]
    netns: bool,

    /// Print the mount namespace inode
    #[arg(long, group = "kind")]
    mntns: bool,

    /// Print the PID namespace inode
    #[arg(long, group = "kind")]
    pidns: bool,

    /// Print the time namespace inode
    #[arg(long, group = "kind")]
    timens: bool,

    /// Print the user namespace inode
    #[arg(long, group = "kind")]
    userns: bool,

    /// Print the UTS namespace inode
    #[arg(long, group = "kind")]
    utsns: bool,

    /// Prefix output with the PID, separated by a colon
    #[arg(short = 'p', long)]
    print_pid: bool,

    /// Process IDs (each may have an optional `:inode` suffix that is ignored)
    #[arg(required = true)]
    pids: Vec<String>,
}

#[derive(Copy, Clone)]
enum Kind {
    Pidfs,
    Ns(&'static str),
}

fn select_kind(a: &Args) -> Kind {
    if a.cgroupns {
        Kind::Ns("cgroup")
    } else if a.ipcns {
        Kind::Ns("ipc")
    } else if a.netns {
        Kind::Ns("net")
    } else if a.mntns {
        Kind::Ns("mnt")
    } else if a.pidns {
        Kind::Ns("pid")
    } else if a.timens {
        Kind::Ns("time")
    } else if a.userns {
        Kind::Ns("user")
    } else if a.utsns {
        Kind::Ns("uts")
    } else {
        Kind::Pidfs
    }
}

fn parse_pid(arg: &str) -> Result<i32, String> {
    let pid_str = arg.split(':').next().unwrap_or(arg);
    pid_str
        .parse::<i32>()
        .map_err(|_| format!("invalid PID: {arg}"))
        .and_then(|p| {
            if p > 0 {
                Ok(p)
            } else {
                Err(format!("invalid PID: {arg}"))
            }
        })
}

fn lookup_inode(pid: i32, kind: Kind) -> Result<u64, String> {
    match kind {
        Kind::Pidfs => {
            let p = Pid::from_raw(pid)
                .ok_or_else(|| format!("invalid PID: {pid}"))?;
            let fd = pidfd_open(p, PidfdFlags::empty()).map_err(|e| {
                format!("pidfd_open({pid}): {}", std::io::Error::from(e))
            })?;
            let st = fstat(&fd)
                .map_err(|e| format!("fstat: {}", std::io::Error::from(e)))?;
            Ok(st.st_ino as u64)
        }
        Kind::Ns(ns) => {
            let path = format!("/proc/{pid}/ns/{ns}");
            let meta =
                std::fs::metadata(&path).map_err(|e| format!("{path}: {e}"))?;
            Ok(meta.ino())
        }
    }
}

pub fn run(args: Args) -> ExitCode {
    let kind = select_kind(&args);
    let mut had_error = false;
    for arg in &args.pids {
        let pid = match parse_pid(arg) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("getino: {e}");
                had_error = true;
                continue;
            }
        };
        match lookup_inode(pid, kind) {
            Ok(ino) => {
                if args.print_pid {
                    println!("{pid}:{ino}");
                } else {
                    println!("{ino}");
                }
            }
            Err(e) => {
                eprintln!("getino: {e}");
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
