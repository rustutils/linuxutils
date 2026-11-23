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
/// It can be used in four ways:
///
///   mount                    List the mounted filesystems, like findmnt.
///   mount -a                 Mount every entry in /etc/fstab that isn't
///                            mounted yet, except noauto, swap and ignore
///                            entries.
///   mount TARGET             Look up TARGET in /etc/fstab and mount it as
///                            configured there.
///   mount SOURCE TARGET      Mount SOURCE on TARGET. SOURCE can be a device
///                            path or a LABEL= or UUID= tag.
///
/// Not supported yet: --move, the --make-shared, --make-slave,
/// --make-private and --make-unbindable options, mount.FSTYPE helpers,
/// loop devices, writing mtab or utab, and -F (--fork).
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX, verbatim_doc_comment)]
#[command(name = "mount", about = "Mount a filesystem")]
pub struct Args {
    /// Mount every entry in fstab. Skips entries with the noauto option,
    /// entries of type swap or ignore, and entries whose target is already
    /// mounted. The -t and -O filters apply.
    #[arg(short = 'a', long = "all", action = ArgAction::SetTrue)]
    all: bool,

    /// Filesystem type, such as ext4, tmpfs or proc. Needed when mounting a
    /// source directly, unless an option like --bind is given. With -a, it
    /// limits which fstab entries are mounted. It takes a comma-separated
    /// list of types to include. If the first type starts with "no", the
    /// list is of types to skip instead (-a -t nonfs,cifs skips nfs and
    /// cifs).
    #[arg(short = 't', long = "types", value_name = "TYPE")]
    fstype: Option<String>,

    /// With -a, only mount entries that match this comma-separated list of
    /// options. Each option must be present (name) or absent (noname), and
    /// an entry must match all of them.
    #[arg(short = 'O', long = "test-opts", value_name = "LIST")]
    test_opts: Option<String>,

    /// Mount options, separated by commas. When the target is looked up in
    /// fstab, these are combined with the options there.
    #[arg(short = 'o', long = "options", value_name = "LIST")]
    options: Option<String>,

    /// Mount read-only (same as -o ro)
    #[arg(short = 'r', long = "read-only", action = ArgAction::SetTrue)]
    read_only: bool,

    /// Mount read-write (same as -o rw, and the default)
    #[arg(short = 'w', long = "rw", action = ArgAction::SetTrue)]
    read_write: bool,

    /// Bind mount (MS_BIND). Needs both a source and a target
    #[arg(short = 'B', long, action = ArgAction::SetTrue)]
    bind: bool,

    /// Recursive bind mount (MS_BIND and MS_REC)
    #[arg(short = 'R', long = "rbind", action = ArgAction::SetTrue)]
    rbind: bool,

    /// Remount an already-mounted filesystem with new options
    #[arg(long, action = ArgAction::SetTrue)]
    remount: bool,

    /// Use FILE as the fstab instead of /etc/fstab
    #[arg(short = 'T', long = "fstab", value_name = "FILE")]
    fstab_file: Option<PathBuf>,

    /// Mount the filesystem with this label (same as a LABEL= source)
    #[arg(short = 'L', long, value_name = "LABEL")]
    label: Option<String>,

    /// Mount the filesystem with this UUID (same as a UUID= source)
    #[arg(short = 'U', long, value_name = "UUID")]
    uuid: Option<String>,

    /// Don't write to /etc/mtab. This has no effect, since mtab is never
    /// written, but is accepted for compatibility
    #[arg(short = 'n', long = "no-mtab", action = ArgAction::SetTrue)]
    no_mtab: bool,

    /// Print what is being done
    #[arg(short = 'v', long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Source to mount, such as a device, an NFS export or a tag. When it
    /// is the only argument, it is treated as the target instead.
    spec: Option<String>,

    /// Mount point.
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

/// Resolve and mount a single fstab entry. Used by both the
/// single-target lookup form and `mount -a`.
fn mount_one_entry(args: &Args, entry: &MountEntry) -> ExitCode {
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
        &entry.target,
        &fstype,
        classified.flags,
        classified.fs_data_arg(),
        args,
    )
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
    mount_one_entry(args, &entry)
}

/// Whether an entry should be skipped by `mount -a` regardless of
/// any `-t`/`-O` filter.
fn skip_for_mount_all(entry: &MountEntry, mounted: &MountTable) -> bool {
    // Userspace marker: don't auto-mount.
    if entry.options.contains("noauto") {
        return true;
    }
    // Swap is for swapon(8); the deprecated `ignore` marker says
    // "leave this alone".
    if entry.fstype == "swap" || entry.fstype == "ignore" {
        return true;
    }
    // Already mounted at this target — skip rather than error.
    if mounted.iter().any(|m| m.target == entry.target) {
        return true;
    }
    false
}

/// Apply a `-t TYPELIST` filter. With a leading `no`-prefixed first
/// token the list is treated as an exclusion set.
fn type_filter_matches(filter: &str, fstype: &str) -> bool {
    let tokens: Vec<&str> =
        filter.split(',').filter(|t| !t.is_empty()).collect();
    if tokens.is_empty() {
        return true;
    }
    let exclude = tokens[0].starts_with("no");
    if exclude {
        let bare: Vec<&str> = tokens
            .iter()
            .map(|t| t.strip_prefix("no").unwrap_or(t))
            .collect();
        !bare.contains(&fstype)
    } else {
        tokens.contains(&fstype)
    }
}

/// Apply a `-O TESTOPTS` filter. Each token is `name` (must be
/// present) or `noname` (must be absent); all ANDed.
fn opts_filter_matches(filter: &str, opts: &OptionList) -> bool {
    for tok in filter.split(',').filter(|t| !t.is_empty()) {
        let (want_present, name) = match tok.strip_prefix("no") {
            Some(rest) => (false, rest),
            None => (true, tok),
        };
        if opts.contains(name) != want_present {
            return false;
        }
    }
    true
}

/// `mount -a` — walk fstab, skip the obvious non-applicable entries,
/// apply -t/-O filters, mount the rest.
fn mount_all(args: &Args) -> ExitCode {
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
    let mounted = match read_mountinfo() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("mount: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut failed = 0u32;
    for entry in fstab.iter() {
        if skip_for_mount_all(entry, &mounted) {
            continue;
        }
        if let Some(f) = args.fstype.as_deref()
            && !type_filter_matches(f, &entry.fstype)
        {
            continue;
        }
        if let Some(f) = args.test_opts.as_deref()
            && !opts_filter_matches(f, &entry.options)
        {
            continue;
        }
        if mount_one_entry(args, entry) != ExitCode::SUCCESS {
            failed += 1;
        }
    }
    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
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
    if args.all {
        if args.spec.is_some()
            || args.target.is_some()
            || args.label.is_some()
            || args.uuid.is_some()
        {
            eprintln!(
                "mount: -a doesn't take a source/target argument or -L/-U"
            );
            return ExitCode::from(2);
        }
        return mount_all(&args);
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn fstab_entry(target: &str, fstype: &str, opts: &str) -> MountEntry {
        MountEntry {
            source: Spec::Path(PathBuf::from("/dev/null")),
            target: PathBuf::from(target),
            fstype: fstype.to_string(),
            options: OptionList::parse(opts),
            ..Default::default()
        }
    }

    fn empty_table() -> MountTable {
        MountTable::new()
    }

    #[test]
    fn skip_noauto() {
        let e = fstab_entry("/mnt", "ext4", "defaults,noauto");
        assert!(skip_for_mount_all(&e, &empty_table()));
    }

    #[test]
    fn skip_swap_fstype() {
        let e = fstab_entry("/swap", "swap", "defaults");
        assert!(skip_for_mount_all(&e, &empty_table()));
    }

    #[test]
    fn skip_ignore_fstype() {
        let e = fstab_entry("/mnt", "ignore", "defaults");
        assert!(skip_for_mount_all(&e, &empty_table()));
    }

    #[test]
    fn skip_already_mounted() {
        let e = fstab_entry("/mnt", "ext4", "defaults");
        let mut mounted = MountTable::new();
        mounted.push(MountEntry {
            target: PathBuf::from("/mnt"),
            ..Default::default()
        });
        assert!(skip_for_mount_all(&e, &mounted));
    }

    #[test]
    fn keep_normal_entry() {
        let e = fstab_entry("/mnt", "ext4", "defaults");
        assert!(!skip_for_mount_all(&e, &empty_table()));
    }

    #[test]
    fn type_filter_allow_list() {
        assert!(type_filter_matches("ext4,xfs", "ext4"));
        assert!(type_filter_matches("ext4,xfs", "xfs"));
        assert!(!type_filter_matches("ext4,xfs", "tmpfs"));
    }

    #[test]
    fn type_filter_exclude_list() {
        // Leading `no` flips the whole list to an exclusion set.
        assert!(!type_filter_matches("nonfs,cifs", "nfs"));
        assert!(!type_filter_matches("nonfs,cifs", "cifs"));
        assert!(type_filter_matches("nonfs,cifs", "ext4"));
    }

    #[test]
    fn type_filter_empty_matches_all() {
        assert!(type_filter_matches("", "ext4"));
    }

    #[test]
    fn opts_filter_must_be_present() {
        let opts = OptionList::parse("ro,nosuid");
        assert!(opts_filter_matches("ro", &opts));
        assert!(!opts_filter_matches("rw", &opts));
    }

    #[test]
    fn opts_filter_must_be_absent() {
        let opts = OptionList::parse("ro,nosuid");
        // `noXXX` here means "XXX must be absent", *not*
        // "literal option named noXXX must be present".
        assert!(opts_filter_matches("norw", &opts));
        assert!(!opts_filter_matches("noro", &opts));
    }

    #[test]
    fn opts_filter_anded() {
        let opts = OptionList::parse("ro,nosuid");
        assert!(opts_filter_matches("ro,nosuid", &opts));
        assert!(!opts_filter_matches("ro,rw", &opts));
        assert!(opts_filter_matches("ro,norw", &opts));
    }

    #[test]
    fn opts_filter_empty_matches_all() {
        let opts = OptionList::parse("ro");
        assert!(opts_filter_matches("", &opts));
    }
}
