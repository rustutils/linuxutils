use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::{
    fd::AsFd,
    fs::{FallocateFlags, Mode, OFlags},
};
use std::{
    io::{Read, Seek, SeekFrom},
    process::ExitCode,
};

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "fallocate",
    about = "Preallocate or deallocate space to a file",
    override_usage = "fallocate [options] <filename>"
)]
pub struct Args {
    /// Length of the range in bytes (required unless --dig-holes)
    #[arg(short, long, value_name = "length")]
    length: Option<String>,

    /// Offset of the range in bytes
    #[arg(short, long, value_name = "offset", default_value = "0")]
    offset: String,

    /// Do not modify the apparent length of the file
    #[arg(short = 'n', long)]
    keep_size: bool,

    /// Remove a byte range from the file without leaving a hole
    #[arg(short = 'c', long, conflicts_with_all = ["punch_hole", "zero_range", "dig_holes", "insert_range", "posix"])]
    collapse_range: bool,

    /// Detect zeroes and replace with holes
    #[arg(short = 'd', long, conflicts_with_all = ["collapse_range", "punch_hole", "zero_range", "insert_range", "posix"])]
    dig_holes: bool,

    /// Report file (sparse) and data (zero-run) holes without
    /// modifying the file.
    #[arg(short = 'r', long, conflicts_with_all = ["collapse_range", "punch_hole", "zero_range", "dig_holes", "insert_range", "posix"])]
    report_holes: bool,

    /// Insert a hole at the specified offset, shifting existing data
    #[arg(short = 'i', long, conflicts_with_all = ["collapse_range", "punch_hole", "zero_range", "dig_holes", "posix"])]
    insert_range: bool,

    /// Deallocate space (punch a hole) in the byte range
    #[arg(short = 'p', long, conflicts_with_all = ["collapse_range", "zero_range", "dig_holes", "insert_range", "posix"])]
    punch_hole: bool,

    /// Zero a byte range in the file
    #[arg(short = 'z', long, conflicts_with_all = ["collapse_range", "punch_hole", "dig_holes", "insert_range", "posix"])]
    zero_range: bool,

    /// Use posix_fallocate(3) compatible operation
    #[arg(short = 'x', long, conflicts_with_all = ["collapse_range", "punch_hole", "zero_range", "dig_holes", "insert_range"])]
    posix: bool,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// The file to operate on
    #[arg(required = true)]
    filename: String,
}

fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty size string".to_string());
    }

    // Find where digits end and suffix begins
    let num_end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());
    let (num_str, suffix) = s.split_at(num_end);
    let base: u64 = num_str
        .parse()
        .map_err(|e| format!("invalid number '{num_str}': {e}"))?;

    let multiplier: u64 = match suffix {
        "" | "B" => 1,
        "K" | "KiB" => 1024,
        "M" | "MiB" => 1024 * 1024,
        "G" | "GiB" => 1024 * 1024 * 1024,
        "T" | "TiB" => 1024u64 * 1024 * 1024 * 1024,
        "P" | "PiB" => 1024u64 * 1024 * 1024 * 1024 * 1024,
        "E" | "EiB" => 1024u64 * 1024 * 1024 * 1024 * 1024 * 1024,
        "KB" => 1000,
        "MB" => 1000 * 1000,
        "GB" => 1000 * 1000 * 1000,
        "TB" => 1000u64 * 1000 * 1000 * 1000,
        "PB" => 1000u64 * 1000 * 1000 * 1000 * 1000,
        "EB" => 1000u64 * 1000 * 1000 * 1000 * 1000 * 1000,
        other => return Err(format!("unknown size suffix '{other}'")),
    };

    base.checked_mul(multiplier)
        .ok_or_else(|| format!("size overflow: {s}"))
}

/// Aggregate hole-tally returned by both `dig_holes` and the data-
/// scan portion of `report_holes`.
#[derive(Default, Clone, Copy)]
struct HoleSummary {
    holes: u64,
    bytes: u64,
}

fn dig_holes(
    fd: &std::fs::File,
    offset: u64,
    length: u64,
) -> Result<HoleSummary, String> {
    let file_size = fd
        .metadata()
        .map_err(|e| format!("failed to get file metadata: {e}"))?
        .len();
    let end = if length == 0 {
        file_size
    } else {
        offset.saturating_add(length).min(file_size)
    };
    let mut pos = offset;
    let mut summary = HoleSummary::default();

    let mut buf = vec![0u8; 64 * 1024];
    let mut reader = std::io::BufReader::new(fd);
    reader
        .seek(SeekFrom::Start(pos))
        .map_err(|e| format!("seek failed: {e}"))?;

    // Track zero runs across chunk boundaries so we punch one big
    // hole per contiguous zero stretch, not one per 64 KiB chunk.
    let mut run_start: Option<u64> = None;

    let mut punch = |start: u64, len: u64| {
        if len < 4096 {
            return;
        }
        let flags = FallocateFlags::PUNCH_HOLE | FallocateFlags::KEEP_SIZE;
        if rustix::fs::fallocate(fd.as_fd(), flags, start, len).is_ok() {
            summary.holes += 1;
            summary.bytes += len;
        }
    };

    while pos < end {
        let to_read = ((end - pos) as usize).min(buf.len());
        let n = reader
            .read(&mut buf[..to_read])
            .map_err(|e| format!("read failed: {e}"))?;
        if n == 0 {
            break;
        }

        let chunk = &buf[..n];
        for (i, &b) in chunk.iter().enumerate() {
            let abs = pos + i as u64;
            if b == 0 {
                if run_start.is_none() {
                    run_start = Some(abs);
                }
            } else if let Some(s) = run_start.take() {
                punch(s, abs - s);
            }
        }

        pos += n as u64;
    }
    if let Some(s) = run_start {
        punch(s, pos - s);
    }

    Ok(summary)
}

/// Produce binary-suffix human-readable size strings matching the
/// upstream `report-holes` output format: `1048576` becomes `1 MiB`,
/// `0` becomes `0 B`.
fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut size = bytes as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    // Whole-number values render without a decimal, fractional
    // values get one decimal place.
    if (size - size.round()).abs() < 0.05 {
        format!("{} {}", size.round() as u64, UNITS[idx])
    } else {
        format!("{size:.1} {}", UNITS[idx])
    }
}

fn pct(numer: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        (numer as f64) / (denom as f64) * 100.0
    }
}

fn format_summary_line(s: HoleSummary, file_size: u64) -> String {
    format!(
        "{} holes, {} ({} bytes, {:.2}% of the file)",
        s.holes,
        human_size(s.bytes),
        s.bytes,
        pct(s.bytes, file_size)
    )
}

/// Walk the file with `lseek(SEEK_HOLE)` / `lseek(SEEK_DATA)` to
/// locate sparse holes, and scan each data section for zero runs
/// big enough to qualify as a candidate "data hole".
fn report_holes(
    fd: &std::fs::File,
    file_size: u64,
) -> Result<(HoleSummary, HoleSummary), String> {
    use rustix::fs::{SeekFrom as RxSeek, seek};
    let mut file_holes = HoleSummary::default();
    let mut data_holes = HoleSummary::default();

    let mut offset: u64 = 0;
    while offset < file_size {
        // Find the next data region.
        let data_start = match seek(fd.as_fd(), RxSeek::Data(offset)) {
            Ok(p) => p,
            Err(e) if e == rustix::io::Errno::NXIO => {
                // No more data. Everything from here to EOF is hole.
                let len = file_size - offset;
                if len > 0 {
                    file_holes.holes += 1;
                    file_holes.bytes += len;
                }
                break;
            }
            Err(e) => return Err(format!("SEEK_DATA failed: {e}")),
        };
        if data_start > offset {
            file_holes.holes += 1;
            file_holes.bytes += data_start - offset;
        }
        // Find where this data region ends.
        let data_end = match seek(fd.as_fd(), RxSeek::Hole(data_start)) {
            Ok(p) => p,
            Err(e) => return Err(format!("SEEK_HOLE failed: {e}")),
        };
        // Scan the data region for zero runs.
        scan_data_for_zero_runs(fd, data_start, data_end, &mut data_holes)?;
        offset = data_end;
    }

    Ok((file_holes, data_holes))
}

fn scan_data_for_zero_runs(
    fd: &std::fs::File,
    start: u64,
    end: u64,
    summary: &mut HoleSummary,
) -> Result<(), String> {
    let mut pos = start;
    let mut buf = vec![0u8; 64 * 1024];
    let mut reader = std::io::BufReader::new(fd);
    reader
        .seek(SeekFrom::Start(pos))
        .map_err(|e| format!("seek failed: {e}"))?;

    // Track zero runs across chunk boundaries so a single 1 MiB
    // zero stretch counts as one hole, not 16.
    let mut run_len: u64 = 0;

    while pos < end {
        let to_read = ((end - pos) as usize).min(buf.len());
        let n = reader
            .read(&mut buf[..to_read])
            .map_err(|e| format!("read failed: {e}"))?;
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];
        let mut i = 0;
        while i < chunk.len() {
            if chunk[i] == 0 {
                run_len += 1;
                i += 1;
            } else {
                if run_len >= 4096 {
                    summary.holes += 1;
                    summary.bytes += run_len;
                }
                run_len = 0;
                i += 1;
            }
        }
        pos += n as u64;
    }
    if run_len >= 4096 {
        summary.holes += 1;
        summary.bytes += run_len;
    }
    Ok(())
}

pub fn run(args: Args) -> ExitCode {
    let offset = match parse_size(&args.offset) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("fallocate: invalid offset: {e}");
            return ExitCode::FAILURE;
        }
    };

    let length = if args.dig_holes || args.report_holes {
        match &args.length {
            Some(s) => match parse_size(s) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("fallocate: invalid length: {e}");
                    return ExitCode::FAILURE;
                }
            },
            // dig-holes and report-holes default to the full file
            None => 0,
        }
    } else {
        match &args.length {
            Some(s) => match parse_size(s) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("fallocate: invalid length: {e}");
                    return ExitCode::FAILURE;
                }
            },
            None => {
                eprintln!("fallocate: required argument --length not provided");
                return ExitCode::FAILURE;
            }
        }
    };

    // Open the file. dig-holes and report-holes operate on an existing
    // file. They don't auto-create one.
    let oflags = if args.dig_holes {
        OFlags::RDWR
    } else if args.report_holes {
        OFlags::RDONLY
    } else {
        OFlags::RDWR | OFlags::CREATE
    };

    let fd = match rustix::fs::open(
        &args.filename,
        oflags,
        Mode::RUSR | Mode::WUSR | Mode::RGRP | Mode::ROTH,
    ) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("fallocate: {}: {e}", args.filename);
            return ExitCode::FAILURE;
        }
    };

    if args.report_holes {
        let file = std::fs::File::from(fd);
        let file_size = match file.metadata() {
            Ok(m) => m.len(),
            Err(e) => {
                eprintln!("fallocate: stat: {e}");
                return ExitCode::FAILURE;
            }
        };
        match report_holes(&file, file_size) {
            Ok((file_h, data_h)) => {
                println!();
                println!("Summary:");
                println!(
                    "file holes: {}",
                    format_summary_line(file_h, file_size)
                );
                println!(
                    "data holes: {}",
                    format_summary_line(data_h, file_size)
                );
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("fallocate: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    if args.dig_holes {
        let file = std::fs::File::from(fd);
        let file_size = match file.metadata() {
            Ok(m) => m.len(),
            Err(e) => {
                eprintln!("fallocate: stat: {e}");
                return ExitCode::FAILURE;
            }
        };
        match dig_holes(&file, offset, length) {
            Ok(summary) => {
                if args.verbose {
                    println!(
                        "{}: {} converted to sparse holes.",
                        args.filename,
                        format_summary_line(summary, file_size)
                    );
                }
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("fallocate: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    let flags = if args.punch_hole {
        FallocateFlags::PUNCH_HOLE | FallocateFlags::KEEP_SIZE
    } else if args.collapse_range {
        FallocateFlags::COLLAPSE_RANGE
    } else if args.insert_range {
        FallocateFlags::INSERT_RANGE
    } else if args.zero_range {
        if args.keep_size {
            FallocateFlags::ZERO_RANGE | FallocateFlags::KEEP_SIZE
        } else {
            FallocateFlags::ZERO_RANGE
        }
    } else if args.keep_size {
        FallocateFlags::KEEP_SIZE
    } else {
        FallocateFlags::empty()
    };

    if args.verbose {
        let mode = if args.punch_hole {
            "punch hole"
        } else if args.collapse_range {
            "collapse range"
        } else if args.insert_range {
            "insert range"
        } else if args.zero_range {
            "zero range"
        } else if args.posix {
            "posix allocate"
        } else {
            "allocate"
        };
        eprintln!(
            "fallocate: {}: {mode} offset {offset}, length {length}",
            args.filename,
        );
    }

    if let Err(e) = rustix::fs::fallocate(fd.as_fd(), flags, offset, length) {
        eprintln!("fallocate: fallocate failed: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_plain() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
    }

    #[test]
    fn parse_size_k() {
        assert_eq!(parse_size("1K").unwrap(), 1024);
    }

    #[test]
    fn parse_size_kib() {
        assert_eq!(parse_size("1KiB").unwrap(), 1024);
    }

    #[test]
    fn parse_size_kb() {
        assert_eq!(parse_size("1KB").unwrap(), 1000);
    }

    #[test]
    fn parse_size_m() {
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
    }

    #[test]
    fn parse_size_mib() {
        assert_eq!(parse_size("1MiB").unwrap(), 1024 * 1024);
    }

    #[test]
    fn parse_size_mb() {
        assert_eq!(parse_size("1MB").unwrap(), 1_000_000);
    }

    #[test]
    fn parse_size_g() {
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn parse_size_invalid_suffix() {
        assert!(parse_size("1X").is_err());
    }

    #[test]
    fn parse_size_empty() {
        assert!(parse_size("").is_err());
    }
}
