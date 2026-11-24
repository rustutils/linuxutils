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

mod mesg {
    use super::parse;
    use linuxutils_term::mesg::Args;

    // ---- Query mode (no positional)

    #[test]
    fn query_no_args() {
        insta::assert_snapshot!(parse::<Args>(&["mesg"]));
    }

    #[test]
    fn query_verbose() {
        insta::assert_snapshot!(parse::<Args>(&["mesg", "-v"]));
    }

    // ---- Set mode

    #[test]
    fn allow_y() {
        insta::assert_snapshot!(parse::<Args>(&["mesg", "y"]));
    }

    #[test]
    fn deny_n() {
        insta::assert_snapshot!(parse::<Args>(&["mesg", "n"]));
    }

    #[test]
    fn allow_y_verbose() {
        insta::assert_snapshot!(parse::<Args>(&["mesg", "-v", "y"]));
    }

    // ---- Error cases

    #[test]
    fn err_invalid_action() {
        insta::assert_snapshot!(parse::<Args>(&["mesg", "yes"]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["mesg", "--bogus"]));
    }
}

mod wall {
    use super::parse;
    use linuxutils_term::wall::Args;

    // ---- SYNOPSIS form: wall [options] [<file> | <message>]

    #[test]
    fn no_args_reads_stdin() {
        insta::assert_snapshot!(parse::<Args>(&["wall"]));
    }

    #[test]
    fn message_inline() {
        insta::assert_snapshot!(parse::<Args>(&[
            "wall",
            "system going down at 5pm",
        ]));
    }

    #[test]
    fn message_multiple_words() {
        insta::assert_snapshot!(parse::<Args>(&[
            "wall", "going", "down", "now",
        ]));
    }

    #[test]
    fn nobanner_flag() {
        insta::assert_snapshot!(parse::<Args>(&["wall", "-n", "hi"]));
    }

    #[test]
    fn timeout_flag() {
        insta::assert_snapshot!(parse::<Args>(&["wall", "-t", "60", "hi",]));
    }

    #[test]
    fn group_flag() {
        insta::assert_snapshot!(parse::<Args>(&["wall", "-g", "wheel", "hi",]));
    }

    #[test]
    fn all_flags_combined() {
        insta::assert_snapshot!(parse::<Args>(&[
            "wall", "-n", "-t", "120", "-g", "wheel", "hi",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_timeout_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["wall", "-t", "abc"]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["wall", "--bogus"]));
    }
}

mod write {
    use super::parse;
    use linuxutils_term::write::Args;

    // ---- SYNOPSIS form: write <user> [<ttyname>]

    #[test]
    fn user_only() {
        insta::assert_snapshot!(parse::<Args>(&["write", "alice"]));
    }

    #[test]
    fn user_and_tty() {
        insta::assert_snapshot!(parse::<Args>(&["write", "alice", "pts/1",]));
    }

    // ---- Error cases

    #[test]
    fn err_no_user() {
        insta::assert_snapshot!(parse::<Args>(&["write"]));
    }

    #[test]
    fn err_too_many_args() {
        insta::assert_snapshot!(parse::<Args>(&[
            "write", "alice", "pts/1", "extra",
        ]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["write", "--bogus"]));
    }
}
