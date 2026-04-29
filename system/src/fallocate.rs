use linuxutils_common::man::ManContent;

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

#[derive(Parser)]
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

fn dig_holes(
    fd: &std::fs::File,
    offset: u64,
    length: u64,
    verbose: bool,
) -> Result<(), String> {
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

    // Use a buffered reader approach: read chunks, find zero regions, punch them
    let mut buf = vec![0u8; 64 * 1024];
    let mut reader = std::io::BufReader::new(fd);
    reader
        .seek(SeekFrom::Start(pos))
        .map_err(|e| format!("seek failed: {e}"))?;

    while pos < end {
        let to_read = ((end - pos) as usize).min(buf.len());
        let n = reader
            .read(&mut buf[..to_read])
            .map_err(|e| format!("read failed: {e}"))?;
        if n == 0 {
            break;
        }

        // Find zero runs in this chunk and punch them
        let chunk = &buf[..n];
        let mut i = 0;
        while i < chunk.len() {
            if chunk[i] == 0 {
                let zero_start = i;
                while i < chunk.len() && chunk[i] == 0 {
                    i += 1;
                }
                let zero_len = i - zero_start;
                // Only punch if the zero run is at least 4K (filesystem block aligned)
                if zero_len >= 4096 {
                    let hole_offset = pos + zero_start as u64;
                    let hole_len = zero_len as u64;
                    let flags =
                        FallocateFlags::PUNCH_HOLE | FallocateFlags::KEEP_SIZE;
                    if let Err(e) = rustix::fs::fallocate(
                        fd.as_fd(),
                        flags,
                        hole_offset,
                        hole_len,
                    ) {
                        if verbose {
                            eprintln!(
                                "fallocate: punch hole at offset {hole_offset}, length {hole_len}: {e}"
                            );
                        }
                    } else if verbose {
                        eprintln!(
                            "fallocate: punched hole at offset {hole_offset}, length {hole_len}"
                        );
                    }
                }
            } else {
                i += 1;
            }
        }

        pos += n as u64;
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

    let length = if args.dig_holes {
        match &args.length {
            Some(s) => match parse_size(s) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("fallocate: invalid length: {e}");
                    return ExitCode::FAILURE;
                }
            },
            None => 0, // dig-holes can work without explicit length (uses file size)
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

    // Open the file
    let oflags = if args.dig_holes {
        OFlags::RDWR
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

    if args.dig_holes {
        let file = std::fs::File::from(fd);
        if let Err(e) = dig_holes(&file, offset, length, args.verbose) {
            eprintln!("fallocate: {e}");
            return ExitCode::FAILURE;
        }
        if args.verbose {
            eprintln!("fallocate: {}: dig holes complete", args.filename);
        }
        return ExitCode::SUCCESS;
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
