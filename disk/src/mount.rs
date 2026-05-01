use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::{ArgAction, Parser};
use cols::{Cols, print_table};
use linuxutils_blkid::Cache as BlkidCache;
use linuxutils_mount::{
    MountEntry, MountFlags, MountTable, OptionList, Spec, Tag, classify,
    syscall::{bind, mount, rbind, remount},
};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::ExitCode,
};

/// Mount a filesystem.
///
/// Three forms:
///   - `mount` (no args) — list active mounts (like `findmnt`).
///   - `mount <target>` — look up the target in `/etc/fstab` and
///     mount it as configured.
///   - `mount <source> <target>` — direct mount. `<source>` may be a
///     device path or a `LABEL=`/`UUID=` tag (resolved via blkid).
///
/// Userspace-only and propagation features deferred to v1: `-a/--all`,
/// `--move`, `--make-{shared,slave,private,unbindable}`,
/// `mount.<fstype>` helpers, loop devices, mtab/utab writing.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "mount", about = "Mount a filesystem")]
pub struct Args {
    /// Filesystem type (e.g. ext4, tmpfs, proc). Required for direct
    /// mounts unless an option like `--bind` is given.
    #[arg(short = 't', long = "types", value_name = "TYPE")]
    fstype: Option<String>,

    /// Mount options (comma-separated). Combined with options from
    /// fstab when looking up a target.
    #[arg(short = 'o', long = "options", value_name = "LIST")]
    options: Option<String>,

    /// Mount read-only (alias for `-o ro`)
    #[arg(short = 'r', long = "read-only", action = ArgAction::SetTrue)]
    read_only: bool,

    /// Mount read-write (alias for `-o rw`, the default)
    #[arg(short = 'w', long = "rw", action = ArgAction::SetTrue)]
    read_write: bool,

    /// Bind-mount (`MS_BIND`); two positional arguments are required
    #[arg(short = 'B', long, action = ArgAction::SetTrue)]
    bind: bool,

    /// Recursive bind (`MS_BIND | MS_REC`)
    #[arg(short = 'R', long = "rbind", action = ArgAction::SetTrue)]
    rbind: bool,

    /// Remount an already-mounted filesystem with new options
    #[arg(long, action = ArgAction::SetTrue)]
    remount: bool,

    /// Use `<file>` as the fstab instead of `/etc/fstab`
    #[arg(short = 'T', long = "fstab", value_name = "FILE")]
    fstab_file: Option<PathBuf>,

    /// Source by label (alias for `<source>` = `LABEL=<label>`)
    #[arg(short = 'L', long, value_name = "LABEL")]
    label: Option<String>,

    /// Source by UUID (alias for `<source>` = `UUID=<uuid>`)
    #[arg(short = 'U', long, value_name = "UUID")]
    uuid: Option<String>,

    /// Don't write to /etc/mtab on success (no-op for us — we don't
    /// write mtab — but accepted for command-line compatibility)
    #[arg(short = 'n', long = "no-mtab", action = ArgAction::SetTrue)]
    no_mtab: bool,

    /// Print what we're doing
    #[arg(short = 'v', long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Source (device, NFS export, tag, ...) — when only one
    /// positional is given, it's interpreted as the target instead.
    spec: Option<String>,

    /// Target mount point.
    target: Option<PathBuf>,
}

#[derive(Cols)]
struct Row {
    #[column(width_fixed = 14)]
    target: String,
    #[column(width_fixed = 14)]
    source: String,
    #[column(width_fixed = 8, header = "FSTYPE")]
    fstype: String,
    #[column(header = "OPTIONS")]
    options: String,
}

fn list_mounts(verbose: bool) -> ExitCode {
    let _ = verbose;
    let table = match read_mountinfo() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("mount: {e}");
            return ExitCode::FAILURE;
        }
    };
    let rows: Vec<Row> = table
        .iter()
        .map(|e| Row {
            target: e.target.to_string_lossy().into_owned(),
            source: render_source(&e.source),
            fstype: e.fstype.clone(),
            options: e.options.to_string(),
        })
        .collect();
    let mut t = Row::to_table(&rows);
    t.name_set("filesystems");
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = print_table(&t, &mut out) {
        eprintln!("mount: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn render_source(s: &Spec) -> String {
    s.to_os_string().to_string_lossy().into_owned()
}

fn read_mountinfo() -> Result<MountTable, String> {
    let f = File::open("/proc/self/mountinfo")
        .map_err(|e| format!("/proc/self/mountinfo: {e}"))?;
    MountTable::parse_mountinfo(BufReader::new(f))
        .map_err(|e| format!("/proc/self/mountinfo: {e}"))
}

fn read_fstab(path: &Path) -> Result<MountTable, String> {
    let f = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    MountTable::parse_fstab(BufReader::new(f))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Resolve a source spec — Path, LABEL=, UUID=, etc. — to the device
/// path the kernel will accept.
fn resolve_source(spec: &Spec) -> Result<PathBuf, String> {
    match spec {
        Spec::Path(p) => Ok(p.clone()),
        Spec::Tag(t) => BlkidCache::new().resolve(t).map_err(|e| {
            format!("can't resolve {}: {e}", t.to_os_string().to_string_lossy())
        }),
    }
}

/// Pick the source string for the kernel: the resolved path turned
/// back into a `&str`. mount(2) accepts arbitrary bytes here, but
/// our wrapper takes &str — for non-UTF-8 device paths the user can
/// fall back to a direct numeric/canonical form.
fn source_arg(p: &Path) -> Result<String, String> {
    p.to_str().map(|s| s.to_string()).ok_or_else(|| {
        format!("source path is not valid UTF-8: {}", p.display())
    })
}

/// Apply -r/-w shortcuts and explicit -o options on top of any
/// fstab options to produce a final OptionList.
fn merge_options(
    fstab_opts: Option<&OptionList>,
    cli_opts: Option<&str>,
    read_only: bool,
    read_write: bool,
) -> OptionList {
    let mut combined = OptionList::new();
    if let Some(o) = fstab_opts {
        for opt in o.iter() {
            combined.push(opt.clone());
        }
    }
    if let Some(s) = cli_opts {
        for opt in OptionList::parse(s).iter() {
            combined.push(opt.clone());
        }
    }
    if read_only {
        combined.push(linuxutils_mount::MountOption::flag("ro"));
    }
    if read_write {
        combined.push(linuxutils_mount::MountOption::flag("rw"));
    }
    combined
}

fn fstab_lookup(fstab: &MountTable, target: &Path) -> Option<MountEntry> {
    fstab.iter().find(|e| e.target == target).cloned()
}

fn do_direct_mount(
    source: &Path,
    target: &Path,
    fstype: &str,
    flags: MountFlags,
    data: Option<&str>,
    args: &Args,
) -> ExitCode {
    if args.verbose {
        eprintln!(
            "mount: {} -> {} type {} flags={:?} data={}",
            source.display(),
            target.display(),
            fstype,
            flags,
            data.unwrap_or("")
        );
    }
    let result = if flags.contains(MountFlags::REMOUNT) {
        remount(target, flags, data)
    } else if flags.contains(MountFlags::BIND | MountFlags::REC) {
        rbind(source, target)
    } else if flags.contains(MountFlags::BIND) {
        bind(source, target)
    } else {
        let src = match source_arg(source) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("mount: {e}");
                return ExitCode::FAILURE;
            }
        };
        mount(Path::new(&src), target, fstype, flags, data)
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "mount: {} -> {}: {e}",
                source.display(),
                target.display()
            );
            ExitCode::FAILURE
        }
    }
}

/// `mount <target>` — fstab lookup form.
fn mount_via_fstab(args: &Args, target: &Path) -> ExitCode {
    let fstab_path = args
        .fstab_file
        .clone()
        .unwrap_or_else(|| PathBuf::from("/etc/fstab"));
    let fstab = match read_fstab(&fstab_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("mount: {e}");
            return ExitCode::FAILURE;
        }
    };
    let entry = match fstab_lookup(&fstab, target) {
        Some(e) => e,
        None => {
            eprintln!(
                "mount: {}: not found in {}",
                target.display(),
                fstab_path.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let source = match resolve_source(&entry.source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("mount: {e}");
            return ExitCode::FAILURE;
        }
    };
    let merged = merge_options(
        Some(&entry.options),
        args.options.as_deref(),
        args.read_only,
        args.read_write,
    );
    let classified = classify(&merged);
    let fstype = args.fstype.clone().unwrap_or(entry.fstype.clone());
    do_direct_mount(
        &source,
        target,
        &fstype,
        classified.flags,
        classified.fs_data_arg(),
        args,
    )
}

/// `mount <source> <target>` — direct form.
fn mount_direct(args: &Args, source_str: &str, target: &Path) -> ExitCode {
    let parsed_source = Spec::parse(source_str);
    let source = match resolve_source(&parsed_source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("mount: {e}");
            return ExitCode::FAILURE;
        }
    };
    let merged = merge_options(
        None,
        args.options.as_deref(),
        args.read_only,
        args.read_write,
    );
    let mut classified = classify(&merged);

    // Bind / rbind / remount can be expressed via -B/-R/--remount
    // flags as well as via -o options; reflect those into the
    // classified.flags so do_direct_mount picks the right syscall.
    if args.bind {
        classified.flags |= MountFlags::BIND;
    }
    if args.rbind {
        classified.flags |= MountFlags::BIND | MountFlags::REC;
    }
    if args.remount {
        classified.flags |= MountFlags::REMOUNT;
    }

    let fstype = match args.fstype.as_deref() {
        Some(t) => t.to_string(),
        None => {
            // Bind / remount don't need a meaningful fstype — kernel
            // ignores it. Direct mount of an unknown fs type is a
            // user error.
            if classified
                .flags
                .intersects(MountFlags::BIND | MountFlags::REMOUNT)
            {
                "none".to_string()
            } else {
                eprintln!(
                    "mount: -t TYPE required for direct mount of {}",
                    source.display()
                );
                return ExitCode::FAILURE;
            }
        }
    };

    do_direct_mount(
        &source,
        target,
        &fstype,
        classified.flags,
        classified.fs_data_arg(),
        args,
    )
}

pub fn run(args: Args) -> ExitCode {
    // -L / -U shortcut into the source slot, so callers can use
    // `mount -L foo /mnt`.
    let source_from_shortcut = args
        .label
        .as_ref()
        .map(|l| Tag::new(linuxutils_mount::TagKind::Label, l).to_os_string())
        .or_else(|| {
            args.uuid.as_ref().map(|u| {
                Tag::new(linuxutils_mount::TagKind::Uuid, u).to_os_string()
            })
        });

    match (
        args.spec.as_deref(),
        args.target.as_deref(),
        source_from_shortcut.as_deref(),
    ) {
        // No positionals at all + no shortcut → list mounts.
        (None, None, None) => list_mounts(args.verbose),

        // Just a target → fstab lookup (`mount /mnt`).
        (Some(spec), None, None) => {
            // The lone positional is the target, not a source — an
            // upstream-compatible quirk.
            mount_via_fstab(&args, Path::new(spec))
        }

        // Source + target both given.
        (Some(spec), Some(target), None) => mount_direct(&args, spec, target),

        // -L / -U shortcut + target.
        (None, Some(target), Some(src_os)) => {
            mount_direct(&args, &src_os.to_string_lossy(), target)
        }

        // -L / -U shortcut alone is ambiguous — needs a target.
        (None, None, Some(_)) => {
            eprintln!("mount: -L/-U requires a target mount point");
            ExitCode::from(2)
        }

        // -L / -U combined with a positional source is conflicting.
        (Some(_), _, Some(_)) => {
            eprintln!(
                "mount: -L/-U can't be combined with a positional source"
            );
            ExitCode::from(2)
        }

        // Target only with no source — unreachable via clap's
        // positional ordering, but the type system insists.
        (None, Some(_), None) => {
            eprintln!("mount: missing source argument");
            ExitCode::from(2)
        }
    }
}
