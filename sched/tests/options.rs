//! Snapshot tests for argument parsing across every tool in this group.
//!
//! Captures each tool's parsed `Args` (via `Debug`) for valid invocations,
//! and the rendered error message for invalid ones. Complements the
//! help-text snapshots in `help.rs` by detecting drift in defaults, field
//! types, and validation rules — things the help text doesn't always show.
//!
//! Test cases mirror the SYNOPSIS forms documented in each tool's man
//! page, with extra cases for non-trivial parsing (size suffixes, mask
//! formats, etc.) and a handful of error paths.
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

mod chrt {
    use super::parse;
    use linuxutils_sched::chrt::Args;

    // ---- SYNOPSIS form 1: chrt [options] <priority> <command> [<arg>...]

    #[test]
    fn fifo_priority_command() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "-f", "10", "ls"]));
    }

    #[test]
    fn rr_priority_command_with_args() {
        insta::assert_snapshot!(parse::<Args>(&[
            "chrt", "-r", "5", "myprog", "--flag", "arg",
        ]));
    }

    #[test]
    fn other_policy_zero_priority() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "-o", "0", "ls"]));
    }

    #[test]
    fn batch_policy() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "-b", "0", "myprog"]));
    }

    #[test]
    fn idle_policy() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "-i", "0", "myprog"]));
    }

    #[test]
    fn deadline_policy_with_params() {
        insta::assert_snapshot!(parse::<Args>(&[
            "chrt", "-d", "-T", "10000", "-P", "100000", "-D", "50000", "0",
            "myprog",
        ]));
    }

    #[test]
    fn reset_on_fork_flag() {
        insta::assert_snapshot!(parse::<Args>(&[
            "chrt", "-R", "-f", "10", "myprog",
        ]));
    }

    // ---- SYNOPSIS form 2: chrt [options] --pid [-p] <priority> <pid>

    #[test]
    fn pid_set_priority() {
        insta::assert_snapshot!(parse::<Args>(&[
            "chrt", "-p", "-f", "10", "1234",
        ]));
    }

    #[test]
    fn pid_set_long_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "chrt", "--pid", "--rr", "5", "1234",
        ]));
    }

    #[test]
    fn pid_set_all_tasks() {
        insta::assert_snapshot!(parse::<Args>(&[
            "chrt", "-a", "-p", "-f", "10", "1234",
        ]));
    }

    // ---- SYNOPSIS form 3: chrt [options] --pid [-p] <pid> (query)

    #[test]
    fn pid_query() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "-p", "1234"]));
    }

    #[test]
    fn pid_query_verbose() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "-v", "-p", "1234"]));
    }

    // ---- Other forms

    #[test]
    fn show_max_priorities() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "-m"]));
    }

    // ---- Error cases

    // `args` is `trailing_var_arg = true, allow_hyphen_values = true`, so
    // unknown flags are eagerly consumed as positionals rather than
    // rejected at parse time. Documented here so the behaviour is
    // load-bearing.
    #[test]
    fn unknown_flag_consumed_as_positional() {
        insta::assert_snapshot!(parse::<Args>(&["chrt", "--bogus"]));
    }

    #[test]
    fn err_sched_runtime_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "chrt",
            "-T",
            "notanumber",
            "-d",
            "0",
            "myprog",
        ]));
    }
}

mod ionice {
    use super::parse;
    use linuxutils_sched::ionice::Args;

    // ---- SYNOPSIS form: ionice (no args, query self)

    #[test]
    fn query_self() {
        insta::assert_snapshot!(parse::<Args>(&["ionice"]));
    }

    // ---- SYNOPSIS form: ionice [-c class] [-n level] [-t] -p PID...

    #[test]
    fn query_pid() {
        insta::assert_snapshot!(parse::<Args>(&["ionice", "-p", "1234"]));
    }

    #[test]
    fn query_multiple_pids() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ionice", "-p", "1234", "5678",
        ]));
    }

    #[test]
    fn set_pid_class_and_level() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ionice", "-c", "1", "-n", "4", "-p", "1234",
        ]));
    }

    #[test]
    fn set_pid_class_by_name() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ionice",
            "-c",
            "best-effort",
            "-n",
            "0",
            "-p",
            "1234",
        ]));
    }

    // ---- SYNOPSIS form: ionice [-c class] [-n level] [-t] -P PGID...

    #[test]
    fn query_pgid() {
        insta::assert_snapshot!(parse::<Args>(&["ionice", "-P", "100"]));
    }

    // ---- SYNOPSIS form: ionice [-c class] [-n level] [-t] -u UID...

    #[test]
    fn query_uid() {
        insta::assert_snapshot!(parse::<Args>(&["ionice", "-u", "1000"]));
    }

    // ---- SYNOPSIS form: ionice [-c class] [-n level] [-t] command [args...]

    #[test]
    fn run_command_with_class() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ionice", "-c", "idle", "myprog",
        ]));
    }

    #[test]
    fn run_command_realtime_high() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ionice", "-c", "realtime", "-n", "0", "myprog", "--flag",
        ]));
    }

    #[test]
    fn ignore_failure_flag() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ionice", "-t", "-c", "1", "-n", "0", "-p", "1234",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["ionice", "--bogus"]));
    }

    #[test]
    fn err_classdata_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(
            &["ionice", "-n", "notanumber",]
        ));
    }
}

mod taskset {
    use super::parse;
    use linuxutils_sched::taskset::Args;

    // ---- SYNOPSIS form 1: taskset [options] [mask] command [arg...]

    #[test]
    fn hex_mask_command() {
        insta::assert_snapshot!(parse::<Args>(&["taskset", "0x3", "ls"]));
    }

    #[test]
    fn hex_mask_multi_byte() {
        insta::assert_snapshot!(parse::<Args>(&[
            "taskset", "0xff", "myprog", "--flag",
        ]));
    }

    #[test]
    fn cpu_list_command() {
        insta::assert_snapshot!(parse::<Args>(&[
            "taskset", "-c", "0,2,4", "myprog",
        ]));
    }

    #[test]
    fn cpu_list_range() {
        insta::assert_snapshot!(parse::<Args>(&[
            "taskset", "-c", "0-3", "myprog",
        ]));
    }

    // ---- SYNOPSIS form 2: taskset [options] -p [mask] pid

    #[test]
    fn pid_query() {
        insta::assert_snapshot!(parse::<Args>(&["taskset", "-p", "1234"]));
    }

    #[test]
    fn pid_set_hex_mask() {
        insta::assert_snapshot!(parse::<Args>(&[
            "taskset", "-p", "0x3", "1234",
        ]));
    }

    #[test]
    fn pid_set_cpu_list_combined_short_flags() {
        insta::assert_snapshot!(parse::<Args>(&[
            "taskset", "-pc", "0-3", "1234",
        ]));
    }

    #[test]
    fn pid_all_tasks() {
        insta::assert_snapshot!(parse::<Args>(&[
            "taskset", "-a", "-p", "0x3", "1234",
        ]));
    }

    // ---- Error cases

    // Same `allow_hyphen_values` behaviour as chrt — see comment there.
    // taskset has no clap-level required args; mask parsing happens at
    // run time, not during clap parsing, so this is the only error-shape
    // worth pinning here.
    #[test]
    fn unknown_flag_consumed_as_positional() {
        insta::assert_snapshot!(parse::<Args>(&["taskset", "--bogus"]));
    }
}
