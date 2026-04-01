use clap::Parser;
use rustix::{
    mount::{self, MountFlags, MountPropagationFlags},
    process,
    thread::UnshareFlags,
};
use std::{
    env,
    ffi::CString,
    fs, io,
    os::unix::process::CommandExt,
    process::{Command, ExitCode},
};

#[derive(Parser)]
#[command(name = "unshare", about = "Run a program in new namespaces")]
pub struct Args {
    /// Unshare mount namespace
    #[arg(short = 'm', long)]
    mount: bool,

    /// Unshare UTS namespace (hostname/domainname)
    #[arg(short = 'u', long)]
    uts: bool,

    /// Unshare IPC namespace
    #[arg(short = 'i', long)]
    ipc: bool,

    /// Unshare network namespace
    #[arg(short = 'n', long)]
    net: bool,

    /// Unshare PID namespace
    #[arg(short = 'p', long)]
    pid: bool,

    /// Unshare user namespace
    #[arg(short = 'U', long)]
    user: bool,

    /// Unshare cgroup namespace
    #[arg(short = 'C', long)]
    cgroup: bool,

    /// Unshare time namespace
    #[arg(short = 'T', long)]
    time: bool,

    /// Fork before executing the program
    #[arg(short = 'f', long)]
    fork: bool,

    /// Map current user to root in user namespace (implies --user)
    #[arg(short = 'r', long = "map-root-user")]
    map_root_user: bool,

    /// Map current user to same UID/GID in user namespace (implies --user)
    #[arg(short = 'c', long = "map-current-user")]
    map_current_user: bool,

    /// Mount /proc filesystem (implies --mount)
    #[arg(long = "mount-proc", num_args = 0..=1, default_missing_value = "/proc")]
    mount_proc: Option<String>,

    /// Set mount propagation (private, shared, slave, unchanged)
    #[arg(long, default_value = "private")]
    propagation: String,

    /// Set root directory
    #[arg(short = 'R', long = "root")]
    root_dir: Option<String>,

    /// Set working directory
    #[arg(short = 'w', long = "wd")]
    work_dir: Option<String>,

    /// Set UID in entered namespace
    #[arg(short = 'S', long = "setuid")]
    set_uid: Option<u32>,

    /// Set GID in entered namespace
    #[arg(short = 'G', long = "setgid")]
    set_gid: Option<u32>,

    /// Allow or deny setgroups in user namespace (allow, deny)
    #[arg(long = "setgroups")]
    setgroups: Option<String>,

    /// Program and arguments to run
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

fn build_unshare_flags(args: &Args) -> UnshareFlags {
    let mut flags = UnshareFlags::empty();
    if args.mount || args.mount_proc.is_some() {
        flags |= UnshareFlags::NEWNS;
    }
    if args.uts {
        flags |= UnshareFlags::NEWUTS;
    }
    if args.ipc {
        flags |= UnshareFlags::NEWIPC;
    }
    if args.net {
        flags |= UnshareFlags::NEWNET;
    }
    if args.pid {
        flags |= UnshareFlags::NEWPID;
    }
    if args.user || args.map_root_user || args.map_current_user {
        flags |= UnshareFlags::NEWUSER;
    }
    if args.cgroup {
        flags |= UnshareFlags::NEWCGROUP;
    }
    if args.time {
        flags |= UnshareFlags::NEWTIME;
    }
    flags
}

fn map_user(
    target_uid: u32,
    target_gid: u32,
    real_uid: u32,
    real_gid: u32,
    deny_setgroups: bool,
) -> io::Result<()> {
    if deny_setgroups {
        fs::write("/proc/self/setgroups", "deny")?;
    }

    fs::write("/proc/self/uid_map", format!("{target_uid} {real_uid} 1\n"))?;
    fs::write("/proc/self/gid_map", format!("{target_gid} {real_gid} 1\n"))?;

    Ok(())
}

fn set_propagation(prop: &str) -> io::Result<()> {
    let flags = match prop {
        "private" => {
            MountPropagationFlags::PRIVATE | MountPropagationFlags::REC
        }
        "shared" => MountPropagationFlags::SHARED | MountPropagationFlags::REC,
        "slave" => {
            MountPropagationFlags::DOWNSTREAM | MountPropagationFlags::REC
        }
        "unchanged" => return Ok(()),
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown propagation type: {other}"),
            ));
        }
    };
    mount::mount_change("/", flags).map_err(io::Error::from)
}

fn do_mount_proc(target: &str) -> io::Result<()> {
    let target_c = CString::new(target)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let proc_c = c"proc";
    mount::mount(proc_c, &*target_c, proc_c, MountFlags::empty(), None)
        .map_err(io::Error::from)
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

fn get_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

fn child_setup(
    root_dir: &Option<String>,
    work_dir: &Option<String>,
    mount_proc: &Option<String>,
    set_gid: Option<u32>,
    set_uid: Option<u32>,
) -> io::Result<()> {
    if let Some(dir) = root_dir {
        std::os::unix::fs::chroot(dir)?;
        env::set_current_dir("/")?;
    }
    if let Some(dir) = work_dir {
        env::set_current_dir(dir)?;
    }
    if let Some(target) = mount_proc {
        do_mount_proc(target)?;
    }
    if let Some(gid) = set_gid {
        do_setgid(gid)?;
    }
    if let Some(uid) = set_uid {
        do_setuid(uid)?;
    }
    Ok(())
}

pub fn run(args: Args) -> ExitCode {
    let flags = build_unshare_flags(&args);

    if flags.is_empty() {
        eprintln!("unshare: no namespaces specified");
        return ExitCode::FAILURE;
    }

    // Capture real UID/GID before unshare, since after unshare(NEWUSER)
    // getuid()/getgid() return the unmapped overflow values (65534).
    let real_uid = process::getuid().as_raw();
    let real_gid = process::getgid().as_raw();

    // SAFETY: We're a single-threaded CLI tool at this point, and we're not
    // using CLONE_FILES which is the main footgun for unshare in threaded programs.
    if let Err(e) = unsafe { rustix::thread::unshare_unsafe(flags) } {
        eprintln!("unshare: unshare failed: {}", io::Error::from(e));
        return ExitCode::FAILURE;
    }

    // Set up user namespace mapping
    if args.map_root_user {
        if let Err(e) = map_user(
            0,
            0,
            real_uid,
            real_gid,
            args.setgroups.as_deref() != Some("allow"),
        ) {
            eprintln!("unshare: failed to map root user: {e}");
            return ExitCode::FAILURE;
        }
    } else if args.map_current_user {
        if let Err(e) = map_user(
            real_uid,
            real_gid,
            real_uid,
            real_gid,
            args.setgroups.as_deref() != Some("allow"),
        ) {
            eprintln!("unshare: failed to map current user: {e}");
            return ExitCode::FAILURE;
        }
    } else if let Some(ref val) = args.setgroups
        && flags.contains(UnshareFlags::NEWUSER)
        && let Err(e) = fs::write("/proc/self/setgroups", val)
    {
        eprintln!("unshare: failed to set setgroups: {e}");
        return ExitCode::FAILURE;
    }

    // Set mount propagation
    if flags.contains(UnshareFlags::NEWNS)
        && args.propagation != "unchanged"
        && let Err(e) = set_propagation(&args.propagation)
    {
        eprintln!("unshare: failed to set propagation: {e}");
        return ExitCode::FAILURE;
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

    if args.fork || args.pid {
        let root_dir = args.root_dir.clone();
        let work_dir = args.work_dir.clone();
        let mount_proc = args.mount_proc.clone();
        let set_uid = args.set_uid;
        let set_gid = args.set_gid;

        let status = unsafe {
            Command::new(&program)
                .args(&program_args)
                .pre_exec(move || {
                    child_setup(
                        &root_dir,
                        &work_dir,
                        &mount_proc,
                        set_gid,
                        set_uid,
                    )
                })
                .status()
        };

        match status {
            Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
            Err(e) => {
                eprintln!("unshare: failed to execute {program}: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        if let Err(e) = child_setup(
            &args.root_dir,
            &args.work_dir,
            &args.mount_proc,
            args.set_gid,
            args.set_uid,
        ) {
            eprintln!("unshare: setup failed: {e}");
            return ExitCode::FAILURE;
        }

        let err = Command::new(&program).args(&program_args).exec();
        eprintln!("unshare: failed to execute {program}: {err}");
        ExitCode::FAILURE
    }
}
