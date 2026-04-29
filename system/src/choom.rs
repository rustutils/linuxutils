use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{fs, os::unix::process::CommandExt, process, process::ExitCode};

#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "choom",
    about = "Display and adjust OOM-killer score",
    override_usage = "choom [options] -p pid\n       \
                      choom [options] -n number -p pid\n       \
                      choom [options] -n number [--] command [args...]"
)]
pub struct Args {
    /// OOM score adjust value to set (-1000 to 1000)
    #[arg(short = 'n', long = "adjust", value_name = "num")]
    adjust: Option<i16>,

    /// Process ID to display or adjust
    #[arg(short = 'p', long, value_name = "num")]
    pid: Option<u32>,

    /// Command and arguments to run with adjusted OOM score
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

fn read_oom(pid: u32) -> std::io::Result<(i32, i16)> {
    let score: i32 = fs::read_to_string(format!("/proc/{pid}/oom_score"))?
        .trim()
        .parse()
        .map_err(std::io::Error::other)?;
    let adj: i16 = fs::read_to_string(format!("/proc/{pid}/oom_score_adj"))?
        .trim()
        .parse()
        .map_err(std::io::Error::other)?;
    Ok((score, adj))
}

fn write_oom_adj(pid: u32, adj: i16) -> std::io::Result<()> {
    fs::write(format!("/proc/{pid}/oom_score_adj"), format!("{adj}\n"))
}

pub fn run(args: Args) -> ExitCode {
    // Validate: needs either -p or a command (but not both).
    let has_command = !args.command.is_empty();

    if args.pid.is_none() && !has_command {
        eprintln!("choom: no PID specified and no command given");
        eprintln!("Try 'choom --help' for more information.");
        return ExitCode::FAILURE;
    }

    if args.pid.is_some() && has_command {
        eprintln!("choom: --pid and command are mutually exclusive");
        return ExitCode::FAILURE;
    }

    if has_command {
        // Set our own OOM adjust value, then exec the command.
        if let Some(adj) = args.adjust
            && let Err(e) = write_oom_adj(std::process::id(), adj)
        {
            eprintln!("choom: failed to set OOM score: {e}");
            return ExitCode::FAILURE;
        }
        let (prog, prog_args) = args.command.split_first().unwrap();
        let err = process::Command::new(prog).args(prog_args).exec();
        eprintln!("choom: {prog}: {err}");
        return ExitCode::FAILURE;
    }

    let pid = args.pid.unwrap();

    if let Some(adj) = args.adjust
        && let Err(e) = write_oom_adj(pid, adj)
    {
        eprintln!("choom: failed to set OOM score for pid {pid}: {e}");
        return ExitCode::FAILURE;
    }

    match read_oom(pid) {
        Ok((score, adj)) => {
            println!("pid {pid}'s current OOM score: {score}");
            println!("pid {pid}'s current OOM score adjust value: {adj}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("choom: failed to read OOM score for pid {pid}: {e}");
            ExitCode::FAILURE
        }
    }
}
