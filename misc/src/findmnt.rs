use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::{ArgAction, Parser};
use cols::{Cols, ColsHeader, OutputMode, print_table};
use linuxutils_mount::{MountEntry, MountTable, Spec};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::ExitCode,
};

/// List or find filesystems by parsing `/proc/self/mountinfo`,
/// `/etc/fstab`, or `/proc/swaps`.
///
/// Reads the kernel's mount table by default; switch with `--fstab`
/// or `--swaps`. Filter by `--source`, `--target`, `--types`, and
/// select columns with `-o`. Tree output (`--tree`) and statfs-derived
/// columns (SIZE/AVAIL/USED/USE%) are not yet implemented.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "findmnt", about = "List active mounts")]
pub struct Args {
    /// Read /etc/fstab instead of /proc/self/mountinfo
    #[arg(short = 's', long, action = ArgAction::SetTrue, group = "src")]
    fstab: bool,

    /// Read /proc/swaps instead of /proc/self/mountinfo
    #[arg(long, action = ArgAction::SetTrue, group = "src")]
    swaps: bool,

    /// Read /proc/self/mountinfo (default; matches `--kernel`)
    #[arg(short = 'k', long, action = ArgAction::SetTrue, group = "src")]
    kernel: bool,

    /// Read mount table from this file instead of the source default
    #[arg(short = 'F', long, value_name = "FILE")]
    tab_file: Option<PathBuf>,

    /// Filter to entries with a matching source (path or LABEL=/UUID=)
    #[arg(short = 'S', long, value_name = "SPEC")]
    source: Option<String>,

    /// Filter to entries with a matching target path
    #[arg(short = 'T', long, value_name = "PATH")]
    target: Option<PathBuf>,

    /// Comma-separated fstype filter; prefix `no` to invert
    /// (`-t nofuse,nofusectl`)
    #[arg(short = 't', long, value_name = "LIST", value_delimiter = ',')]
    types: Option<Vec<String>>,

    /// Use JSON output format
    #[arg(short = 'J', long, action = ArgAction::SetTrue)]
    json: bool,

    /// Use raw output format
    #[arg(short = 'r', long, action = ArgAction::SetTrue)]
    raw: bool,

    /// Don't print headings
    #[arg(short = 'n', long, action = ArgAction::SetTrue)]
    noheadings: bool,

    /// Define which output columns to use
    #[arg(short = 'o', long, value_delimiter = ',', value_name = "LIST")]
    output: Option<Vec<String>>,

    /// Output all available columns
    #[arg(long, action = ArgAction::SetTrue)]
    output_all: bool,

    /// Positional fallback: a target path or source spec, equivalent
    /// to `--target` or `--source` respectively. Matches upstream
    /// behaviour where a bare argument is interpreted as either.
    #[arg(value_name = "DEVICE-OR-MOUNTPOINT")]
    spec: Option<String>,
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

    #[column(header = "VFS-OPTIONS")]
    vfs_options: String,

    #[column(header = "FS-OPTIONS")]
    fs_options: String,

    #[column(width_fixed = 10, header = "FSROOT")]
    fsroot: String,

    #[column(width_fixed = 7, header = "MAJ:MIN")]
    majmin: String,

    #[column(header = "OPT-FIELDS")]
    opt_fields: String,

    #[column(right, width_fixed = 4)]
    freq: String,

    #[column(right, width_fixed = 6)]
    passno: String,
}

fn pick_header(name: &str) -> Option<RowHeader> {
    RowHeader::all()
        .iter()
        .find(|h| h.name().eq_ignore_ascii_case(name))
        .copied()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Mountinfo,
    Fstab,
    Swaps,
}

impl Source {
    fn default_path(self) -> &'static Path {
        match self {
            Source::Mountinfo => Path::new("/proc/self/mountinfo"),
            Source::Fstab => Path::new("/etc/fstab"),
            Source::Swaps => Path::new("/proc/swaps"),
        }
    }

    fn parse(self, path: &Path) -> std::io::Result<MountTable> {
        let f = File::open(path)?;
        let r = BufReader::new(f);
        match self {
            Source::Mountinfo => MountTable::parse_mountinfo(r),
            Source::Fstab => MountTable::parse_fstab(r),
            Source::Swaps => MountTable::parse_swaps(r),
        }
    }
}

fn select_source(args: &Args) -> Source {
    if args.fstab {
        Source::Fstab
    } else if args.swaps {
        Source::Swaps
    } else {
        Source::Mountinfo
    }
}

fn render_source(s: &Spec) -> String {
    s.to_os_string().to_string_lossy().into_owned()
}

fn matches_filters(e: &MountEntry, args: &Args) -> bool {
    // --source filter: exact match against the rendered spec form,
    // so both `/dev/sda1` and `LABEL=foo` work.
    if let Some(src) = &args.source {
        if &render_source(&e.source) != src {
            return false;
        }
    }
    // Positional argument: when it starts with `/`, treat as target
    // path; otherwise treat as source spec. (Matches the upstream
    // shortcut.)
    if let Some(spec) = &args.spec {
        let looks_like_path = spec.starts_with('/');
        if looks_like_path {
            if e.target != Path::new(spec) {
                return false;
            }
        } else if &render_source(&e.source) != spec {
            return false;
        }
    }
    if let Some(t) = &args.target {
        if &e.target != t {
            return false;
        }
    }
    if let Some(types) = &args.types {
        // Positive types must match at least one; `no<fs>` entries
        // must NOT match. Mix like upstream:
        // `-t ext4,noproc` ⇒ ext4-only, but excluding proc.
        let mut positive: Vec<&str> = Vec::new();
        let mut negative: Vec<&str> = Vec::new();
        for t in types {
            if let Some(rest) = t.strip_prefix("no") {
                negative.push(rest);
            } else {
                positive.push(t);
            }
        }
        if !positive.is_empty() && !positive.iter().any(|t| e.fstype == *t) {
            return false;
        }
        if negative.iter().any(|t| e.fstype == *t) {
            return false;
        }
    }
    true
}

fn make_row(e: &MountEntry) -> Row {
    let majmin = e
        .devno
        .map(|(maj, min)| format!("{maj}:{min}"))
        .unwrap_or_default();
    let fsroot = e
        .root
        .as_deref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let opt_fields = e.optional_fields.clone().unwrap_or_default();
    let vfs_options = e
        .vfs_options
        .as_ref()
        .map(|o| o.to_string())
        .unwrap_or_default();
    let fs_options = e
        .fs_options
        .as_ref()
        .map(|o| o.to_string())
        .unwrap_or_default();
    let freq = e.freq.map(|n| n.to_string()).unwrap_or_default();
    let passno = e.passno.map(|n| n.to_string()).unwrap_or_default();
    Row {
        target: e.target.to_string_lossy().into_owned(),
        source: render_source(&e.source),
        fstype: e.fstype.clone(),
        options: e.options.to_string(),
        vfs_options,
        fs_options,
        fsroot,
        majmin,
        opt_fields,
        freq,
        passno,
    }
}

const DEFAULT_HEADERS: &[&str] = &["TARGET", "SOURCE", "FSTYPE", "OPTIONS"];

fn select_headers(args: &Args) -> Result<Vec<RowHeader>, String> {
    if args.output_all {
        return Ok(RowHeader::all().to_vec());
    }
    if let Some(names) = &args.output {
        let mut hs = Vec::with_capacity(names.len());
        for name in names {
            match pick_header(name.trim()) {
                Some(h) => hs.push(h),
                None => return Err(format!("unknown column: {name}")),
            }
        }
        return Ok(hs);
    }
    Ok(DEFAULT_HEADERS
        .iter()
        .map(|n| pick_header(n).expect("default header missing"))
        .collect())
}

pub fn run(args: Args) -> ExitCode {
    let source = select_source(&args);
    let path = args
        .tab_file
        .clone()
        .unwrap_or_else(|| source.default_path().to_path_buf());

    let table = match source.parse(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("findmnt: {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };

    let headers = match select_headers(&args) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("findmnt: {e}");
            return ExitCode::FAILURE;
        }
    };

    let rows: Vec<Row> = table
        .iter()
        .filter(|e| matches_filters(e, &args))
        .map(make_row)
        .collect();

    let mut table = Row::to_table_with(&rows, &headers);
    table.name_set("filesystems");
    if args.json {
        table.output_mode_set(OutputMode::Json);
    } else if args.raw {
        table.output_mode_set(OutputMode::Raw);
    }
    if args.noheadings {
        table.headings_set(false);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = print_table(&table, &mut out) {
        eprintln!("findmnt: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
