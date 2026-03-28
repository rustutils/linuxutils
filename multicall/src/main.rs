use clap::{Command, CommandFactory as _, FromArgMatches as _};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cmd = Command::new("rustutils-util-linux")
        .multicall(true)
        .subcommand(rustutils_colrm::Args::command())
        .subcommand(rustutils_line::Args::command())
        .subcommand(rustutils_look::Args::command())
        .subcommand(rustutils_mcookie::Args::command())
        .subcommand(rustutils_nologin::Args::command())
        .subcommand(rustutils_rev::Args::command());

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("colrm", m)) => rustutils_colrm::run(
            rustutils_colrm::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("line", m)) => rustutils_line::run(
            rustutils_line::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("look", m)) => rustutils_look::run(
            rustutils_look::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("mcookie", m)) => rustutils_mcookie::run(
            rustutils_mcookie::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("nologin", m)) => rustutils_nologin::run(
            rustutils_nologin::Args::from_arg_matches(m).unwrap(),
        ),
        Some(("rev", m)) => rustutils_rev::run(
            rustutils_rev::Args::from_arg_matches(m).unwrap(),
        ),
        _ => unreachable!("clap ensures a valid subcommand"),
    }
}
