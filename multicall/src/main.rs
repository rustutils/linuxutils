use clap::{Command, CommandFactory as _, FromArgMatches as _};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cmd = Command::new("rustutils-util-linux")
        .multicall(true)
        .subcommand(rustutils_cal::Args::command())
        .subcommand(rustutils_colrm::Args::command())
        .subcommand(rustutils_line::Args::command())
        .subcommand(rustutils_look::Args::command())
        .subcommand(rustutils_lsirq::Args::command())
        .subcommand(rustutils_lsmem::Args::command())
        .subcommand(rustutils_mcookie::Args::command())
        .subcommand(rustutils_mesg::Args::command())
        .subcommand(rustutils_mountpoint::Args::command())
        .subcommand(rustutils_nologin::Args::command())
        .subcommand(rustutils_setpgid::Args::command())
        .subcommand(rustutils_setsid::Args::command())
        .subcommand(rustutils_rev::Args::command())
        .subcommand(rustutils_uuidgen::Args::command())
        .subcommand(rustutils_uuidparse::Args::command());

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("cal", m)) => rustutils_cal::run(
            rustutils_cal::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("colrm", m)) => rustutils_colrm::run(
            rustutils_colrm::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("line", m)) => rustutils_line::run(
            rustutils_line::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("look", m)) => rustutils_look::run(
            rustutils_look::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("lsirq", m)) => rustutils_lsirq::run(
            rustutils_lsirq::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("lsmem", m)) => rustutils_lsmem::run(
            rustutils_lsmem::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("mcookie", m)) => rustutils_mcookie::run(
            rustutils_mcookie::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("mesg", m)) => rustutils_mesg::run(
            rustutils_mesg::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("mountpoint", m)) => rustutils_mountpoint::run(
            rustutils_mountpoint::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("nologin", m)) => rustutils_nologin::run(
            rustutils_nologin::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("setpgid", m)) => rustutils_setpgid::run(
            rustutils_setpgid::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("setsid", m)) => rustutils_setsid::run(
            rustutils_setsid::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("rev", m)) => rustutils_rev::run(
            rustutils_rev::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("uuidgen", m)) => rustutils_uuidgen::run(
            rustutils_uuidgen::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("uuidparse", m)) => rustutils_uuidparse::run(
            rustutils_uuidparse::Args::from_arg_matches(m).unwrap(),
        ),
        _ => unreachable!("clap ensures a valid subcommand"),
    }
}
