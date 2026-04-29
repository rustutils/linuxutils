use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::{
    fd::BorrowedFd,
    fs::{self, Advice},
};
use std::{fs::File, os::unix::io::AsRawFd, process::ExitCode};

/// Advise the kernel about file access patterns via posix_fadvise(2).
///
/// Most commonly used with `dontneed` to drop a file's pages from the
/// page cache.
#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "fadvise",
    about = "Advise the kernel about file access patterns"
)]
pub struct Args {
    /// Advice to apply: normal, sequential, random, noreuse, willneed, dontneed
    #[arg(short = 'a', long, default_value = "dontneed")]
    advice: String,

    /// Beginning offset in bytes
    #[arg(short = 'o', long, default_value = "0")]
    offset: u64,

    /// Length of range in bytes (0 = to end of file)
    #[arg(short = 'l', long, default_value = "0")]
    length: u64,

    /// Operate on an already-open file descriptor
    #[arg(short = 'd', long = "fd")]
    file_descriptor: Option<i32>,

    /// File to advise on
    filename: Option<String>,
}

fn parse_advice(s: &str) -> Result<Advice, String> {
    match s {
        "normal" => Ok(Advice::Normal),
        "sequential" => Ok(Advice::Sequential),
        "random" => Ok(Advice::Random),
        "noreuse" => Ok(Advice::NoReuse),
        "willneed" => Ok(Advice::WillNeed),
        "dontneed" => Ok(Advice::DontNeed),
        other => Err(format!("unknown advice: {other}")),
    }
}

pub fn run(args: Args) -> ExitCode {
    let advice = match parse_advice(&args.advice) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("fadvise: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Some(raw_fd) = args.file_descriptor {
        let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
        if let Err(e) = fs::fadvise(
            fd,
            args.offset,
            std::num::NonZeroU64::new(args.length),
            advice,
        ) {
            eprintln!("fadvise: fd {raw_fd}: {}", std::io::Error::from(e));
            return ExitCode::FAILURE;
        }
    } else if let Some(ref path) = args.filename {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fadvise: {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        let fd = unsafe { BorrowedFd::borrow_raw(file.as_raw_fd()) };
        if let Err(e) = fs::fadvise(
            fd,
            args.offset,
            std::num::NonZeroU64::new(args.length),
            advice,
        ) {
            eprintln!("fadvise: {path}: {}", std::io::Error::from(e));
            return ExitCode::FAILURE;
        }
    } else {
        eprintln!("fadvise: no filename or file descriptor specified");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
