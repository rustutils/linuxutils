use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{io, process::ExitCode};

/// Create System V IPC resources.
///
/// Creates shared memory segments, message queues, or semaphore arrays
/// using shmget(2), msgget(2), or semget(2) with IPC_PRIVATE.
#[derive(Parser)]
#[command(name = "ipcmk", about = "Create System V IPC resources")]
pub struct Args {
    /// Create shared memory segment of given size in bytes
    #[arg(short = 'M', long = "shmem")]
    shmem: Option<usize>,

    /// Create message queue
    #[arg(short = 'Q', long)]
    queue: bool,

    /// Create semaphore array with N elements
    #[arg(short = 'S', long)]
    semaphore: Option<i32>,

    /// Permissions in octal (default: 0644)
    #[arg(short = 'p', long, default_value = "0644")]
    mode: String,
}

fn parse_mode(s: &str) -> Result<i32, String> {
    i32::from_str_radix(s, 8).map_err(|_| format!("invalid mode: {s}"))
}

pub fn run(args: Args) -> ExitCode {
    let mode = match parse_mode(&args.mode) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("ipcmk: {e}");
            return ExitCode::FAILURE;
        }
    };

    let flags = libc::IPC_CREAT | libc::IPC_EXCL | mode;

    if let Some(size) = args.shmem {
        let id = unsafe { libc::shmget(libc::IPC_PRIVATE, size, flags) };
        if id < 0 {
            eprintln!("ipcmk: shmget: {}", io::Error::last_os_error());
            return ExitCode::FAILURE;
        }
        println!("Shared memory id: {id}");
    }

    if args.queue {
        let id = unsafe { libc::msgget(libc::IPC_PRIVATE, flags) };
        if id < 0 {
            eprintln!("ipcmk: msgget: {}", io::Error::last_os_error());
            return ExitCode::FAILURE;
        }
        println!("Message queue id: {id}");
    }

    if let Some(nsems) = args.semaphore {
        let id = unsafe { libc::semget(libc::IPC_PRIVATE, nsems, flags) };
        if id < 0 {
            eprintln!("ipcmk: semget: {}", io::Error::last_os_error());
            return ExitCode::FAILURE;
        }
        println!("Semaphore id: {id}");
    }

    if args.shmem.is_none() && !args.queue && args.semaphore.is_none() {
        eprintln!("ipcmk: specify -M, -Q, or -S. Try 'ipcmk --help'");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
