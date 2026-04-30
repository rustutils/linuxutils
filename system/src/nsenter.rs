use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::{
    fd::AsFd,
    fs::{self, Mode, OFlags},
    thread::{LinkNameSpaceType, move_into_link_name_space},
};
use std::{
    env, io,
    os::unix::process::CommandExt,
    process::{Command, ExitCode},
};

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "nsenter", about = "Run a program in different namespaces")]
pub struct Args {
    /// Target process to get namespaces from
    #[arg(short = 't', long = "target")]
    target: Option<u32>,

    /// Enter all namespaces of the target process
    #[arg(short = 'a', long)]
    all: bool,

    /// Enter mount namespace
    #[arg(short = 'm', long, num_args = 0..=1, default_missing_value = "")]
    mount: Option<String>,

    /// Enter UTS namespace
    #[arg(short = 'u', long, num_args = 0..=1, default_missing_value = "")]
    uts: Option<String>,

    /// Enter IPC namespace
    #[arg(short = 'i', long, num_args = 0..=1, default_missing_value = "")]
    ipc: Option<String>,

    /// Enter network namespace
    #[arg(short = 'n', long, num_args = 0..=1, default_missing_value = "")]
    net: Option<String>,

    /// Enter PID namespace
    #[arg(short = 'p', long, num_args = 0..=1, default_missing_value = "")]
    pid: Option<String>,

    /// Enter user namespace
    #[arg(short = 'U', long, num_args = 0..=1, default_missing_value = "")]
    user: Option<String>,

    /// Enter cgroup namespace
    #[arg(short = 'C', long, num_args = 0..=1, default_missing_value = "")]
    cgroup: Option<String>,

    /// Enter time namespace
    #[arg(short = 'T', long, num_args = 0..=1, default_missing_value = "")]
    time: Option<String>,

    /// Don't fork before exec (default is to fork when entering PID namespace)
    #[arg(short = 'F', long = "no-fork")]
    no_fork: bool,

    /// Don't modify UID/GID when entering user namespace
    #[arg(long = "preserve-credentials")]
    preserve_credentials: bool,

    /// Set root directory
    #[arg(short = 'r', long = "root", num_args = 0..=1, default_missing_value = "")]
    root_dir: Option<String>,

    /// Set working directory
    #[arg(short = 'w', long = "wd", num_args = 0..=1, default_missing_value = "")]
    work_dir: Option<String>,

    /// Set UID in entered namespace
    #[arg(short = 'S', long = "setuid")]
    set_uid: Option<u32>,

    /// Set GID in entered namespace
    #[arg(short = 'G', long = "setgid")]
    set_gid: Option<u32>,

    /// Program and arguments to run
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

struct NsEntry {
    path: String,
    ns_type: LinkNameSpaceType,
}

const NS_TYPES: &[(&str, LinkNameSpaceType)] = &[
    ("mnt", LinkNameSpaceType::Mount),
    ("uts", LinkNameSpaceType::HostNameAndNISDomainName),
    ("ipc", LinkNameSpaceType::InterProcessCommunication),
    ("net", LinkNameSpaceType::Network),
    ("pid", LinkNameSpaceType::ProcessID),
    ("user", LinkNameSpaceType::User),
    ("cgroup", LinkNameSpaceType::ControlGroup),
    ("time", LinkNameSpaceType::Time),
];

fn ns_path(pid: u32, ns: &str) -> String {
    format!("/proc/{pid}/ns/{ns}")
}

fn resolve_ns_file(
    explicit: &Option<String>,
    pid: Option<u32>,
    ns_name: &str,
) -> Option<String> {
    match explicit {
        Some(path) if !path.is_empty() => Some(path.clone()),
        Some(_) => pid.map(|p| ns_path(p, ns_name)),
        None => None,
    }
}

fn enter_namespace(path: &str, ns_type: LinkNameSpaceType) -> io::Result<()> {
    let fd = fs::open(path, OFlags::RDONLY, Mode::empty())
        .map_err(io::Error::from)?;
    move_into_link_name_space(fd.as_fd(), Some(ns_type))
        .map_err(io::Error::from)
}

fn get_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

fn do_setuid(uid: u32) -> io::Result<()> {
    if unsafe { libc::setuid(uid) } != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn do_setgid(gid: u32) -> io::Result<()> {
    if unsafe { libc::setgid(gid) } != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn run(args: Args) -> ExitCode {
    let pid = args.target;

    // Build list of namespaces to enter
    let mut entries: Vec<NsEntry> = Vec::new();

    if args.all {
        if let Some(pid) = pid {
            for &(ns_name, ns_type) in NS_TYPES {
                let path = ns_path(pid, ns_name);
                if std::path::Path::new(&path).exists() {
                    entries.push(NsEntry { path, ns_type });
                }
            }
        } else {
            eprintln!("nsenter: --all requires --target");
            return ExitCode::FAILURE;
        }
    }

    // Individual namespace flags override --all entries
    let ns_opts: &[(&Option<String>, &str, LinkNameSpaceType)] = &[
        (&args.user, "user", LinkNameSpaceType::User),
        (&args.mount, "mnt", LinkNameSpaceType::Mount),
        (
            &args.uts,
            "uts",
            LinkNameSpaceType::HostNameAndNISDomainName,
        ),
        (
            &args.ipc,
            "ipc",
            LinkNameSpaceType::InterProcessCommunication,
        ),
        (&args.net, "net", LinkNameSpaceType::Network),
        (&args.pid, "pid", LinkNameSpaceType::ProcessID),
        (&args.cgroup, "cgroup", LinkNameSpaceType::ControlGroup),
        (&args.time, "time", LinkNameSpaceType::Time),
    ];

    for &(opt, ns_name, ns_type) in ns_opts {
        if let Some(path) = resolve_ns_file(opt, pid, ns_name) {
            entries.retain(|e| e.ns_type != ns_type);
            entries.push(NsEntry { path, ns_type });
        }
    }

    if entries.is_empty() {
        eprintln!("nsenter: no namespaces specified");
        return ExitCode::FAILURE;
    }

    // Enter user namespace first (if present) to gain capabilities
    let has_user_ns =
        entries.iter().any(|e| e.ns_type == LinkNameSpaceType::User);
    if has_user_ns {
        let idx = entries
            .iter()
            .position(|e| e.ns_type == LinkNameSpaceType::User)
            .unwrap();
        let entry = entries.remove(idx);
        if let Err(e) = enter_namespace(&entry.path, entry.ns_type) {
            eprintln!(
                "nsenter: failed to enter user namespace ({}): {e}",
                entry.path
            );
            return ExitCode::FAILURE;
        }

        if !args.preserve_credentials {
            let uid = args.set_uid.unwrap_or(0);
            let gid = args.set_gid.unwrap_or(0);
            let _ = do_setgid(gid);
            let _ = do_setuid(uid);
        }
    }

    // Enter remaining namespaces
    for entry in &entries {
        if let Err(e) = enter_namespace(&entry.path, entry.ns_type) {
            eprintln!(
                "nsenter: failed to enter namespace ({}): {e}",
                entry.path
            );
            return ExitCode::FAILURE;
        }
    }

    // Set root/working directory
    if let Some(ref dir) = args.root_dir {
        let dir = if dir.is_empty() {
            pid.map(|p| format!("/proc/{p}/root"))
                .unwrap_or_else(|| "/".to_string())
        } else {
            dir.clone()
        };
        if let Err(e) = std::os::unix::fs::chroot(&dir) {
            eprintln!("nsenter: chroot to {dir} failed: {e}");
            return ExitCode::FAILURE;
        }
        if let Err(e) = env::set_current_dir("/") {
            eprintln!("nsenter: chdir failed: {e}");
            return ExitCode::FAILURE;
        }
    }
    if let Some(ref dir) = args.work_dir {
        let dir = if dir.is_empty() {
            pid.map(|p| format!("/proc/{p}/cwd"))
                .unwrap_or_else(|| ".".to_string())
        } else {
            dir.clone()
        };
        if let Err(e) = env::set_current_dir(&dir) {
            eprintln!("nsenter: chdir to {dir} failed: {e}");
            return ExitCode::FAILURE;
        }
    }

    // Set UID/GID if specified (and not already done for user namespace)
    if !has_user_ns {
        if let Some(gid) = args.set_gid
            && let Err(e) = do_setgid(gid)
        {
            eprintln!("nsenter: setgid failed: {e}");
            return ExitCode::FAILURE;
        }
        if let Some(uid) = args.set_uid
            && let Err(e) = do_setuid(uid)
        {
            eprintln!("nsenter: setuid failed: {e}");
            return ExitCode::FAILURE;
        }
    }

    let program = if args.command.is_empty() {
        get_shell()
    } else {
        args.command[0].clone()
    };
    let program_args: Vec<&str> = if args.command.len() > 1 {
        args.command[1..].iter().map(|s| s.as_str()).collect()
    } else {
        vec![]
    };

    let has_pid_ns = entries
        .iter()
        .any(|e| e.ns_type == LinkNameSpaceType::ProcessID);

    if has_pid_ns && !args.no_fork {
        match Command::new(&program).args(&program_args).status() {
            Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
            Err(e) => {
                eprintln!("nsenter: failed to execute {program}: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        let err = Command::new(&program).args(&program_args).exec();
        eprintln!("nsenter: failed to execute {program}: {err}");
        ExitCode::FAILURE
    }
}
