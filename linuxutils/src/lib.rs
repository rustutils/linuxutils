//! The table of every tool in linuxutils.
//!
//! The multicall binary dispatches through [`TOOLS`], and `xtask mangen`
//! generates a man page for each entry, so a tool only needs to be added
//! here once.

use clap::{ArgMatches, Command, CommandFactory as _, FromArgMatches as _};
use linuxutils_common::man::ManContent;
use std::process::ExitCode;

/// One tool that the multicall binary can dispatch to.
pub struct Tool {
    /// Builds the tool's clap command. Its name is the tool's name.
    pub command: fn() -> Command,
    /// Parses the matches into the tool's arguments and runs it.
    pub run: fn(&ArgMatches) -> ExitCode,
    /// Hand-written man page content for the tool.
    pub man: &'static ManContent,
}

/// Builds the `TOOLS` table from a list of `crate::module` paths. Each
/// module must export `Args` (a clap parser), `run(Args) -> ExitCode` and
/// `MAN`.
macro_rules! tools {
    ($($krate:ident :: $tool:ident),* $(,)?) => {
        /// Every tool, in the order they are listed in.
        pub const TOOLS: &[Tool] = &[$(
            Tool {
                // Plain path rather than `<Args as CommandFactory>`, because
                // some tools (whereis) define inherent `command` and
                // `from_arg_matches` methods around a private clap struct.
                command: $krate::$tool::Args::command,
                run: |m| $krate::$tool::run(
                    $krate::$tool::Args::from_arg_matches(m).unwrap_or_else(|e| e.exit()),
                ),
                man: &$krate::$tool::MAN,
            },
        )*];
    };
}

tools! {
    linuxutils_disk::blockdev,
    linuxutils_disk::isosize,
    linuxutils_disk::mount,
    linuxutils_disk::umount,
    linuxutils_login::nologin,
    linuxutils_misc::blkid,
    linuxutils_misc::cal,
    linuxutils_misc::copyfilerange,
    linuxutils_misc::exch,
    linuxutils_misc::fincore,
    linuxutils_misc::findfs,
    linuxutils_misc::findmnt,
    linuxutils_misc::getino,
    linuxutils_misc::getopt,
    linuxutils_misc::kill,
    linuxutils_misc::logger,
    linuxutils_misc::look,
    linuxutils_misc::lslocks,
    linuxutils_misc::mcookie,
    linuxutils_misc::namei,
    linuxutils_misc::rename,
    linuxutils_misc::uuidgen,
    linuxutils_misc::uuidparse,
    linuxutils_misc::waitpid,
    linuxutils_misc::whereis,
    linuxutils_sched::chrt,
    linuxutils_sched::ionice,
    linuxutils_sched::taskset,
    linuxutils_system::blkdiscard,
    linuxutils_system::choom,
    linuxutils_system::ctrlaltdel,
    linuxutils_system::dmesg,
    linuxutils_system::eject,
    linuxutils_system::fadvise,
    linuxutils_system::fallocate,
    linuxutils_system::flock,
    linuxutils_system::fsfreeze,
    linuxutils_system::fstrim,
    linuxutils_system::ipcmk,
    linuxutils_system::ipcrm,
    linuxutils_system::ipcs,
    linuxutils_system::losetup,
    linuxutils_system::lscpu,
    linuxutils_system::lsipc,
    linuxutils_system::lsirq,
    linuxutils_system::lsmem,
    linuxutils_system::lsns,
    linuxutils_system::mountpoint,
    linuxutils_system::nsenter,
    linuxutils_system::pipesz,
    linuxutils_system::pivot_root,
    linuxutils_system::prlimit,
    linuxutils_system::renice,
    linuxutils_system::rfkill,
    linuxutils_system::setarch,
    linuxutils_system::setpgid,
    linuxutils_system::setsid,
    linuxutils_system::swapoff,
    linuxutils_system::swapon,
    linuxutils_system::unshare,
    linuxutils_system::wdctl,
    linuxutils_term::mesg,
    linuxutils_term::wall,
    linuxutils_term::write,
    linuxutils_text::bits,
    linuxutils_text::colrm,
    linuxutils_text::column,
    linuxutils_text::hexdump,
    linuxutils_text::line,
    linuxutils_text::rev,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;
    use std::{collections::BTreeSet, path::Path};

    /// Group crates whose `pub mod` entries are all tools.
    const GROUP_CRATES: &[&str] = &[
        "disk", "kmod", "login", "misc", "sched", "system", "term", "text",
    ];

    /// Every tool module in a group crate must be in `TOOLS`, and the
    /// other way round. Catches a new tool that was never wired in here.
    #[test]
    fn tools_match_group_crates() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut modules = BTreeSet::new();
        for krate in GROUP_CRATES {
            let lib =
                std::fs::read_to_string(root.join(krate).join("src/lib.rs"))
                    .unwrap();
            for line in lib.lines() {
                if let Some(rest) = line.trim().strip_prefix("pub mod ") {
                    modules
                        .insert(rest.trim_end_matches(';').trim().to_string());
                }
            }
        }

        let tools: BTreeSet<String> = TOOLS
            .iter()
            .map(|t| (t.command)().get_name().to_string())
            .collect();

        assert_eq!(modules, tools);
    }

    #[test]
    fn command_is_valid() {
        Command::new("linuxutils")
            .subcommands(TOOLS.iter().map(|t| (t.command)()))
            .debug_assert();
    }
}
