use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs::{self, File},
    io::{self, BufRead},
    os::unix::io::AsRawFd,
    process::ExitCode,
};

/// Copy byte ranges between files using copy_file_range(2).
///
/// Performs kernel-space file copies that can create reflinked files
/// when the filesystem supports it. Range format: `src_off:dst_off:length`.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "copyfilerange", about = "Copy byte ranges between files")]
pub struct Args {
    /// Read range specs from file (one per line)
    #[arg(short = 'r', long = "ranges")]
    ranges_file: Option<String>,

    /// Print each range as it's copied
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Source file
    source: String,

    /// Destination file
    destination: String,

    /// Range specs: source_offset:destination_offset:length
    #[arg(trailing_var_arg = true)]
    ranges: Vec<String>,
}

struct Range {
    src_off: u64,
    dst_off: u64,
    length: u64,
}

fn parse_range(
    spec: &str,
    last_src: u64,
    last_dst: u64,
) -> Result<Range, String> {
    let parts: Vec<&str> = spec.splitn(3, ':').collect();

    let src_off = if parts.is_empty() || parts[0].is_empty() {
        last_src
    } else {
        parse_size(parts[0])?
    };

    let dst_off = if parts.len() < 2 || parts[1].is_empty() {
        last_dst
    } else {
        parse_size(parts[1])?
    };

    let length = if parts.len() < 3 || parts[2].is_empty() {
        0
    } else {
        parse_size(parts[2])?
    };

    Ok(Range {
        src_off,
        dst_off,
        length,
    })
}

fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(0);
    }
    // Support simple K/M/G suffixes
    let (num_str, mult) = if let Some(n) = s.strip_suffix('G') {
        (n, 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix('M') {
        (n, 1024 * 1024)
    } else if let Some(n) = s.strip_suffix('K') {
        (n, 1024)
    } else {
        (s, 1)
    };
    num_str
        .parse::<u64>()
        .map(|n| n * mult)
        .map_err(|_| format!("invalid number: {s}"))
}

fn do_copy_file_range(
    src_fd: i32,
    src_off: &mut i64,
    dst_fd: i32,
    dst_off: &mut i64,
    len: usize,
) -> io::Result<usize> {
    let ret = unsafe {
        libc::copy_file_range(src_fd, src_off, dst_fd, dst_off, len, 0)
    };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}

pub fn run(args: Args) -> ExitCode {
    let src = match File::open(&args.source) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("copyfilerange: {}: {e}", args.source);
            return ExitCode::FAILURE;
        }
    };

    let dst = match File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&args.destination)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("copyfilerange: {}: {e}", args.destination);
            return ExitCode::FAILURE;
        }
    };

    let src_size = fs::metadata(&args.source).map(|m| m.len()).unwrap_or(0);

    // Collect ranges
    let mut range_specs: Vec<String> = args.ranges.clone();
    if let Some(ref path) = args.ranges_file {
        match File::open(path) {
            Ok(f) => {
                for line in io::BufReader::new(f).lines().map_while(Result::ok)
                {
                    let line = line.trim().to_string();
                    if !line.is_empty() {
                        range_specs.push(line);
                    }
                }
            }
            Err(e) => {
                eprintln!("copyfilerange: {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    if range_specs.is_empty() {
        eprintln!("copyfilerange: no ranges specified");
        return ExitCode::FAILURE;
    }

    let mut last_src: u64 = 0;
    let mut last_dst: u64 = 0;

    for spec in &range_specs {
        let range = match parse_range(spec, last_src, last_dst) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("copyfilerange: invalid range '{spec}': {e}");
                return ExitCode::FAILURE;
            }
        };

        let copy_len = if range.length == 0 {
            src_size.saturating_sub(range.src_off)
        } else {
            range.length
        };

        let mut src_off = range.src_off as i64;
        let mut dst_off = range.dst_off as i64;
        let mut remaining = copy_len as usize;

        while remaining > 0 {
            let chunk = remaining.min(1 << 30); // 1 GiB max per call
            match do_copy_file_range(
                src.as_raw_fd(),
                &mut src_off,
                dst.as_raw_fd(),
                &mut dst_off,
                chunk,
            ) {
                Ok(0) => break,
                Ok(n) => remaining -= n,
                Err(e) => {
                    eprintln!("copyfilerange: copy failed: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }

        if args.verbose {
            let copied = copy_len as usize - remaining;
            eprintln!(
                "{}:{} -> {}:{} ({copied} bytes)",
                args.source, range.src_off, args.destination, range.dst_off
            );
        }

        last_src = src_off as u64;
        last_dst = dst_off as u64;
    }

    ExitCode::SUCCESS
}
