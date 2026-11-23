use clap::{Arg, ArgAction, Command};
use linuxutils::TOOLS;
use std::{io::Write as _, process::ExitCode};

fn tool_subcommands(cmd: Command) -> Command {
    cmd.subcommands(TOOLS.iter().map(|t| (t.command)()))
}

fn main() -> ExitCode {
    // Invoked as a tool name (argv[0] is a symlink such as `rev`), or as
    // `linuxutils <tool> ...`.
    let dispatcher = Command::new("linuxutils")
        .about("Run any linuxutils tool, by name or through a symlink")
        .arg(
            Arg::new("list")
                .long("list")
                .action(ArgAction::SetTrue)
                .exclusive(true)
                .help("Print the name of every tool, one per line"),
        )
        .args_conflicts_with_subcommands(true)
        .arg_required_else_help(true);
    let cmd = tool_subcommands(Command::new("linuxutils").multicall(true))
        .subcommand(tool_subcommands(dispatcher));

    let matches = cmd.get_matches();
    let (name, m) = match matches.subcommand() {
        Some(("linuxutils", m)) => match m.subcommand() {
            Some(sub) => sub,
            None => {
                // Only --list gets here, since an argument is required.
                let mut out = std::io::stdout().lock();
                for tool in TOOLS {
                    // Stop quietly if stdout is closed, as with `| head`.
                    if writeln!(out, "{}", (tool.command)().get_name()).is_err()
                    {
                        break;
                    }
                }
                return ExitCode::SUCCESS;
            }
        },
        Some((name, m)) => (name, m),
        None => unreachable!("clap ensures a valid subcommand"),
    };

    let tool = TOOLS
        .iter()
        .find(|t| (t.command)().get_name() == name)
        .expect("clap only matches known subcommands");
    (tool.run)(m)
}
