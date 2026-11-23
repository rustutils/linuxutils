use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use cols::{Cols, ColsHeader, OutputMode, print_table};
use std::{fs::File, os::unix::io::AsRawFd, process::ExitCode};

/// Count pages of file contents in core, using mincore(2).
///
/// Each file is mapped into memory (MAP_PRIVATE and MAP_NORESERVE) to
/// find which of its pages are in the page cache. The number of bytes
/// in the cache is shown next to the file's size.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "fincore", about = "Count pages of file contents in core")]
pub struct Args {
    /// Print SIZE in bytes rather than human-readable format
    #[arg(short, long)]
    bytes: bool,

    /// Use JSON output format
    #[arg(short = 'J', long)]
    json: bool,

    /// Don't print headings
    #[arg(short = 'n', long)]
    noheadings: bool,

    /// Define which output columns to use
    #[arg(short = 'o', long, value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Output all available columns
    #[arg(long)]
    output_all: bool,

    /// Use the raw output format
    #[arg(short = 'r', long)]
    raw: bool,

    /// Files to inspect
    #[arg(required = true)]
    files: Vec<String>,
}

#[derive(Cols)]
struct Row {
    #[column(right, width_fixed = 6)]
    res: String,

    #[column(right, width_fixed = 6)]
    pages: String,

    #[column(right, width_fixed = 6)]
    size: String,

    file: String,
}

fn pick_header(name: &str) -> Option<RowHeader> {
    RowHeader::all()
        .iter()
        .find(|h| h.name().eq_ignore_ascii_case(name))
        .copied()
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
    let mut val = bytes as f64;
    let mut unit = 0;
    while val >= 1024.0 && unit + 1 < UNITS.len() {
        val /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes}B")
    } else if val >= 10.0 {
        format!("{:.0}{}", val, UNITS[unit])
    } else {
        format!("{:.1}{}", val, UNITS[unit])
    }
}

struct Stats {
    file: String,
    size: u64,
    pages_in_core: u64,
}

fn fincore_one(path: &str, page_size: usize) -> Result<Stats, String> {
    let f = File::open(path).map_err(|e| format!("{path}: {e}"))?;
    let meta = f.metadata().map_err(|e| format!("{path}: {e}"))?;

    if !meta.is_file() {
        return Err(format!("{path}: not a regular file"));
    }

    let len = meta.len();
    if len == 0 {
        return Ok(Stats {
            file: path.to_string(),
            size: 0,
            pages_in_core: 0,
        });
    }

    let len_usize = match usize::try_from(len) {
        Ok(n) => n,
        Err(_) => return Err(format!("{path}: file too large to map")),
    };

    let addr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len_usize,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_NORESERVE,
            f.as_raw_fd(),
            0,
        )
    };
    if addr == libc::MAP_FAILED {
        return Err(format!(
            "{path}: mmap: {}",
            std::io::Error::last_os_error()
        ));
    }

    let n_pages = len_usize.div_ceil(page_size);
    let mut vec = vec![0u8; n_pages];

    let rc = unsafe { libc::mincore(addr, len_usize, vec.as_mut_ptr()) };
    let _ = unsafe { libc::munmap(addr, len_usize) };

    if rc != 0 {
        return Err(format!(
            "{path}: mincore: {}",
            std::io::Error::last_os_error()
        ));
    }

    let pages_in_core = vec.iter().filter(|&&b| b & 1 != 0).count() as u64;
    Ok(Stats {
        file: path.to_string(),
        size: len,
        pages_in_core,
    })
}

fn make_row(stats: &Stats, page_size: u64, bytes: bool) -> Row {
    let res = stats.pages_in_core * page_size;
    let fmt = |n: u64| if bytes { n.to_string() } else { human_size(n) };
    Row {
        res: fmt(res),
        pages: stats.pages_in_core.to_string(),
        size: fmt(stats.size),
        file: stats.file.clone(),
    }
}

pub fn run(args: Args) -> ExitCode {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size <= 0 {
        eprintln!("fincore: cannot determine page size");
        return ExitCode::FAILURE;
    }
    let page_size = page_size as u64;

    let headers: Option<Vec<RowHeader>> = match (args.output_all, &args.output)
    {
        (true, _) => None,
        (false, None) => None,
        (false, Some(names)) => {
            let mut hs = Vec::with_capacity(names.len());
            for name in names {
                match pick_header(name.trim()) {
                    Some(h) => hs.push(h),
                    None => {
                        eprintln!("fincore: unknown column: {name}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            Some(hs)
        }
    };

    let mut rows = Vec::with_capacity(args.files.len());
    let mut had_error = false;
    for path in &args.files {
        match fincore_one(path, page_size as usize) {
            Ok(s) => rows.push(make_row(&s, page_size, args.bytes)),
            Err(e) => {
                eprintln!("fincore: {e}");
                had_error = true;
            }
        }
    }

    let mut table = match headers {
        Some(hs) => Row::to_table_with(&rows, &hs),
        None => Row::to_table(&rows),
    };
    table.name_set("fincore");
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
        eprintln!("fincore: {e}");
        return ExitCode::FAILURE;
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
