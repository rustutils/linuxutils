//! Snapshot tests for argument parsing across every tool in this group.
//!
//! Captures each tool's parsed `Args` (via `Debug`) for valid invocations,
//! and the rendered error message for invalid ones. Complements the
//! help-text snapshots in `help.rs` by detecting drift in defaults, field
//! types, and validation rules: things the help text doesn't always show.
//!
//! Run `cargo insta review` after intentional CLI changes to accept the
//! updated snapshots.

use clap::Parser;
use std::fmt::Debug;

fn parse<P: Parser + Debug>(argv: &[&str]) -> String {
    match P::try_parse_from(argv) {
        Ok(v) => format!("{v:#?}"),
        Err(e) => format!("ERROR:\n{e}"),
    }
}

mod nologin {
    use super::parse;
    use linuxutils_login::nologin::Args;

    // nologin's flags are all `hide = true` shell-compatibility shims
    // that the runtime ignores. The snapshots pin them so the
    // accept-and-ignore behaviour stays load-bearing. Invocations from
    // /etc/passwd-style shell entries must keep parsing.

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["nologin"]));
    }

    #[test]
    fn shell_compat_command_flag() {
        insta::assert_snapshot!(parse::<Args>(&["nologin", "-c", "echo hi",]));
    }

    #[test]
    fn shell_compat_login_flag() {
        insta::assert_snapshot!(parse::<Args>(&["nologin", "-l"]));
    }

    #[test]
    fn shell_compat_init_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "nologin",
            "--init-file",
            "/tmp/init",
        ]));
    }

    #[test]
    fn shell_compat_combined() {
        insta::assert_snapshot!(parse::<Args>(&[
            "nologin",
            "--login",
            "--noprofile",
            "--norc",
            "--posix",
            "--restricted",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["nologin", "--bogus"]));
    }
}
