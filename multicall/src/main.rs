use clap::{ArgMatches, Command, CommandFactory as _, FromArgMatches as _};
use std::process::ExitCode;

fn tool_subcommands(cmd: Command) -> Command {
    cmd.subcommand(linuxutils_cal::Args::command())
        .subcommand(linuxutils_choom::Args::command())
        .subcommand(linuxutils_chrt::Args::command())
        .subcommand(linuxutils_colrm::Args::command())
        .subcommand(linuxutils_dmesg::Args::command())
        .subcommand(linuxutils_column::Args::command())
        .subcommand(linuxutils_fallocate::Args::command())
        .subcommand(linuxutils_flock::Args::command())
        .subcommand(linuxutils_hexdump::Args::command())
        .subcommand(linuxutils_ionice::Args::command())
        .subcommand(linuxutils_isosize::Args::command())
        .subcommand(linuxutils_kill::Args::command())
        .subcommand(linuxutils_line::Args::command())
        .subcommand(linuxutils_logger::Args::command())
        .subcommand(linuxutils_look::Args::command())
        .subcommand(linuxutils_lsirq::Args::command())
        .subcommand(linuxutils_lslocks::Args::command())
        .subcommand(linuxutils_lsmem::Args::command())
        .subcommand(linuxutils_lsns::Args::command())
        .subcommand(linuxutils_mcookie::Args::command())
        .subcommand(linuxutils_mesg::Args::command())
        .subcommand(linuxutils_namei::Args::command())
        .subcommand(linuxutils_mountpoint::Args::command())
        .subcommand(linuxutils_nologin::Args::command())
        .subcommand(linuxutils_prlimit::Args::command())
        .subcommand(linuxutils_rename::Args::command())
        .subcommand(linuxutils_renice::Args::command())
        .subcommand(linuxutils_setpgid::Args::command())
        .subcommand(linuxutils_setsid::Args::command())
        .subcommand(linuxutils_taskset::Args::command())
        .subcommand(linuxutils_rev::Args::command())
        .subcommand(linuxutils_rfkill::Args::command())
        .subcommand(linuxutils_uuidgen::Args::command())
        .subcommand(linuxutils_uuidparse::Args::command())
        .subcommand(linuxutils_waitpid::Args::command())
        .subcommand(linuxutils_whereis::Args::command())
}

fn dispatch(name: &str, m: &ArgMatches) -> ExitCode {
    match name {
        "cal" => linuxutils_cal::run(
            linuxutils_cal::Args::from_arg_matches(m).unwrap(),
        ),
        "choom" => linuxutils_choom::run(
            linuxutils_choom::Args::from_arg_matches(m).unwrap(),
        ),
        "chrt" => linuxutils_chrt::run(
            linuxutils_chrt::Args::from_arg_matches(m).unwrap(),
        ),
        "colrm" => linuxutils_colrm::run(
            linuxutils_colrm::Args::from_arg_matches(m).unwrap(),
        ),
        "dmesg" => linuxutils_dmesg::run(
            linuxutils_dmesg::Args::from_arg_matches(m).unwrap(),
        ),
        "column" => linuxutils_column::run(
            linuxutils_column::Args::from_arg_matches(m).unwrap(),
        ),
        "fallocate" => linuxutils_fallocate::run(
            linuxutils_fallocate::Args::from_arg_matches(m).unwrap(),
        ),
        "flock" => linuxutils_flock::run(
            linuxutils_flock::Args::from_arg_matches(m).unwrap(),
        ),
        "hexdump" => linuxutils_hexdump::run(
            linuxutils_hexdump::Args::from_arg_matches(m).unwrap(),
        ),
        "ionice" => linuxutils_ionice::run(
            linuxutils_ionice::Args::from_arg_matches(m).unwrap(),
        ),
        "isosize" => linuxutils_isosize::run(
            linuxutils_isosize::Args::from_arg_matches(m).unwrap(),
        ),
        "kill" => linuxutils_kill::run(
            linuxutils_kill::Args::from_arg_matches(m).unwrap(),
        ),
        "line" => linuxutils_line::run(
            linuxutils_line::Args::from_arg_matches(m).unwrap(),
        ),
        "logger" => linuxutils_logger::run(
            linuxutils_logger::Args::from_arg_matches(m).unwrap(),
        ),
        "look" => linuxutils_look::run(
            linuxutils_look::Args::from_arg_matches(m).unwrap(),
        ),
        "lsirq" => linuxutils_lsirq::run(
            linuxutils_lsirq::Args::from_arg_matches(m).unwrap(),
        ),
        "lslocks" => linuxutils_lslocks::run(
            linuxutils_lslocks::Args::from_arg_matches(m).unwrap(),
        ),
        "lsmem" => linuxutils_lsmem::run(
            linuxutils_lsmem::Args::from_arg_matches(m).unwrap(),
        ),
        "lsns" => linuxutils_lsns::run(
            linuxutils_lsns::Args::from_arg_matches(m).unwrap(),
        ),
        "mcookie" => linuxutils_mcookie::run(
            linuxutils_mcookie::Args::from_arg_matches(m).unwrap(),
        ),
        "mesg" => linuxutils_mesg::run(
            linuxutils_mesg::Args::from_arg_matches(m).unwrap(),
        ),
        "namei" => linuxutils_namei::run(
            linuxutils_namei::Args::from_arg_matches(m).unwrap(),
        ),
        "mountpoint" => linuxutils_mountpoint::run(
            linuxutils_mountpoint::Args::from_arg_matches(m).unwrap(),
        ),
        "nologin" => linuxutils_nologin::run(
            linuxutils_nologin::Args::from_arg_matches(m).unwrap(),
        ),
        "prlimit" => linuxutils_prlimit::run(
            linuxutils_prlimit::Args::from_arg_matches(m).unwrap(),
        ),
        "rename" => linuxutils_rename::run(
            linuxutils_rename::Args::from_arg_matches(m).unwrap(),
        ),
        "renice" => linuxutils_renice::run(
            linuxutils_renice::Args::from_arg_matches(m).unwrap(),
        ),
        "setpgid" => linuxutils_setpgid::run(
            linuxutils_setpgid::Args::from_arg_matches(m).unwrap(),
        ),
        "setsid" => linuxutils_setsid::run(
            linuxutils_setsid::Args::from_arg_matches(m).unwrap(),
        ),
        "taskset" => linuxutils_taskset::run(
            linuxutils_taskset::Args::from_arg_matches(m).unwrap(),
        ),
        "rev" => linuxutils_rev::run(
            linuxutils_rev::Args::from_arg_matches(m).unwrap(),
        ),
        "rfkill" => linuxutils_rfkill::run(
            linuxutils_rfkill::Args::from_arg_matches(m).unwrap(),
        ),
        "uuidgen" => linuxutils_uuidgen::run(
            linuxutils_uuidgen::Args::from_arg_matches(m).unwrap(),
        ),
        "uuidparse" => linuxutils_uuidparse::run(
            linuxutils_uuidparse::Args::from_arg_matches(m).unwrap(),
        ),
        "waitpid" => linuxutils_waitpid::run(
            linuxutils_waitpid::Args::from_arg_matches(m).unwrap(),
        ),
        "whereis" => linuxutils_whereis::run(
            linuxutils_whereis::Args::from_arg_matches(m).unwrap(),
        ),
        _ => unreachable!("clap ensures a valid subcommand"),
    }
}

fn main() -> ExitCode {
    let cmd = Command::new("linuxutils").multicall(true);
    let cmd = tool_subcommands(cmd);
    let cmd = cmd.subcommand(tool_subcommands(
        Command::new("linuxutils").subcommand_required(true),
    ));

    let matches = cmd.get_matches();

    let (name, m) = match matches.subcommand() {
        Some(("linuxutils", m)) => m.subcommand().unwrap(),
        Some(other) => other,
        None => unreachable!("clap ensures a valid subcommand"),
    };

    dispatch(name, m)
}
