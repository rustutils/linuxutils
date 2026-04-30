//! Snapshot tests for argument parsing across every tool in this group.
//!
//! Captures each tool's parsed `Args` (via `Debug`) for valid invocations,
//! and the rendered error message for invalid ones. Complements the
//! help-text snapshots in `help.rs` by detecting drift in defaults, field
//! types, and validation rules — things the help text doesn't always show.
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

mod cal {
    use super::parse;
    use linuxutils_misc::cal::Args;

    // ---- SYNOPSIS form: cal [options] [[[day] month] year]

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["cal"]));
    }

    #[test]
    fn year_only() {
        insta::assert_snapshot!(parse::<Args>(&["cal", "2025"]));
    }

    #[test]
    fn month_year() {
        insta::assert_snapshot!(parse::<Args>(&["cal", "4", "2025"]));
    }

    #[test]
    fn day_month_year() {
        insta::assert_snapshot!(parse::<Args>(&[
            "cal", "15", "4", "2025",
        ]));
    }

    #[test]
    fn three_months() {
        insta::assert_snapshot!(parse::<Args>(&["cal", "-3"]));
    }

    #[test]
    fn n_months() {
        insta::assert_snapshot!(parse::<Args>(&[
            "cal", "-n", "12", "2025",
        ]));
    }

    #[test]
    fn full_year_twelve_columns() {
        insta::assert_snapshot!(parse::<Args>(&[
            "cal", "-Y", "-c", "4", "2025",
        ]));
    }

    #[test]
    fn julian_monday_first() {
        insta::assert_snapshot!(parse::<Args>(&["cal", "-j", "-m"]));
    }

    // ---- Error cases

    #[test]
    fn err_columns_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["cal", "-c", "abc"]));
    }
}

mod copyfilerange {
    use super::parse;
    use linuxutils_misc::copyfilerange::Args;

    // ---- SYNOPSIS form: copyfilerange [options] <source> <dest> [ranges...]

    #[test]
    fn basic_two_files() {
        insta::assert_snapshot!(parse::<Args>(&[
            "copyfilerange", "src", "dst",
        ]));
    }

    #[test]
    fn with_inline_ranges() {
        insta::assert_snapshot!(parse::<Args>(&[
            "copyfilerange",
            "src",
            "dst",
            "0:0:4096",
            "4096:4096:4096",
        ]));
    }

    #[test]
    fn with_ranges_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "copyfilerange",
            "-r",
            "ranges.txt",
            "src",
            "dst",
        ]));
    }

    #[test]
    fn verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "copyfilerange", "-v", "src", "dst",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_dest() {
        insta::assert_snapshot!(parse::<Args>(&["copyfilerange", "src"]));
    }

    #[test]
    fn err_no_args() {
        insta::assert_snapshot!(parse::<Args>(&["copyfilerange"]));
    }
}

mod fincore {
    use super::parse;
    use linuxutils_misc::fincore::Args;

    // ---- SYNOPSIS form: fincore [options] file...

    #[test]
    fn single_file() {
        insta::assert_snapshot!(parse::<Args>(&["fincore", "/etc/hostname"]));
    }

    #[test]
    fn multiple_files() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fincore", "a.bin", "b.bin", "c.bin",
        ]));
    }

    #[test]
    fn json_bytes() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fincore", "-J", "-b", "f",
        ]));
    }

    #[test]
    fn raw_noheadings() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fincore", "--raw", "--noheadings", "f",
        ]));
    }

    #[test]
    fn output_columns() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fincore", "-o", "RES,PAGES,SIZE,FILE", "f",
        ]));
    }

    #[test]
    fn output_all() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fincore",
            "--output-all",
            "f",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_files() {
        insta::assert_snapshot!(parse::<Args>(&["fincore"]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["fincore", "--bogus", "f"]));
    }
}

mod getopt {
    use super::parse;
    use linuxutils_misc::getopt::Args;

    // ---- SYNOPSIS form 1: getopt optstring parameters

    #[test]
    fn classic_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "getopt", "ab:c", "-a", "-b", "x", "-c",
        ]));
    }

    // ---- SYNOPSIS form 2: getopt [options] [--] optstring parameters

    #[test]
    fn with_options_dash_dash() {
        insta::assert_snapshot!(parse::<Args>(&[
            "getopt", "-q", "--", "ab:", "-a",
        ]));
    }

    // ---- SYNOPSIS form 3: getopt [options] -o optstring [--] parameters

    #[test]
    fn with_long_options() {
        insta::assert_snapshot!(parse::<Args>(&[
            "getopt",
            "-o",
            "ab:c",
            "-l",
            "alpha,beta:,gamma",
            "--",
            "--alpha",
            "--beta=x",
        ]));
    }

    #[test]
    fn alternative_long_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "getopt",
            "-a",
            "-o",
            "x",
            "-l",
            "longopt",
            "--",
            "-longopt",
        ]));
    }

    #[test]
    fn shell_csh() {
        insta::assert_snapshot!(parse::<Args>(&[
            "getopt", "-s", "csh", "-o", "ab", "--", "-a", "-b",
        ]));
    }

    #[test]
    fn name_for_errors() {
        insta::assert_snapshot!(parse::<Args>(&[
            "getopt", "-n", "myprog", "-o", "ab", "--", "-a",
        ]));
    }

    #[test]
    fn test_mode() {
        insta::assert_snapshot!(parse::<Args>(&["getopt", "-T"]));
    }

    #[test]
    fn unquoted_output() {
        insta::assert_snapshot!(parse::<Args>(&[
            "getopt", "-u", "-o", "a", "--", "-a",
        ]));
    }
}

mod kill {
    use super::parse;
    use linuxutils_misc::kill::Args;

    // kill uses `args: Vec<String>` with `trailing_var_arg = true,
    // allow_hyphen_values = true` so EVERYTHING is accepted by clap and
    // dispatched at run time. Like chrt and taskset, an "unknown flag"
    // can't be caught by clap; it's just consumed as a positional.

    // ---- SYNOPSIS form 1: kill [-s signal | -signal] pid...

    #[test]
    fn default_signal() {
        insta::assert_snapshot!(parse::<Args>(&["kill", "1234"]));
    }

    #[test]
    fn dash_s_signal() {
        insta::assert_snapshot!(parse::<Args>(&[
            "kill", "-s", "TERM", "1234",
        ]));
    }

    #[test]
    fn dash_signal_name() {
        insta::assert_snapshot!(parse::<Args>(&["kill", "-HUP", "1234"]));
    }

    #[test]
    fn dash_signal_number() {
        insta::assert_snapshot!(parse::<Args>(&["kill", "-9", "1234"]));
    }

    #[test]
    fn multiple_pids() {
        insta::assert_snapshot!(parse::<Args>(&[
            "kill", "-TERM", "1234", "5678",
        ]));
    }

    // ---- SYNOPSIS form 2: kill -l [number]

    #[test]
    fn list_all_signals() {
        insta::assert_snapshot!(parse::<Args>(&["kill", "-l"]));
    }

    #[test]
    fn list_specific_signal() {
        insta::assert_snapshot!(parse::<Args>(&["kill", "-l", "9"]));
    }

    // ---- SYNOPSIS form 3: kill -L

    #[test]
    fn list_table() {
        insta::assert_snapshot!(parse::<Args>(&["kill", "-L"]));
    }
}

mod logger {
    use super::parse;
    use linuxutils_misc::logger::Args;

    #[test]
    fn no_message_reads_stdin() {
        insta::assert_snapshot!(parse::<Args>(&["logger"]));
    }

    #[test]
    fn inline_message() {
        insta::assert_snapshot!(parse::<Args>(&[
            "logger", "hello", "world",
        ]));
    }

    #[test]
    fn priority_facility_level() {
        insta::assert_snapshot!(parse::<Args>(&[
            "logger", "-p", "daemon.warning", "boot complete",
        ]));
    }

    #[test]
    fn tag_and_pid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "logger", "-t", "myapp", "-i", "starting",
        ]));
    }

    #[test]
    fn read_from_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "logger", "-f", "log.txt", "-e",
        ]));
    }

    #[test]
    fn stderr_and_socket() {
        insta::assert_snapshot!(parse::<Args>(&[
            "logger", "-s", "-u", "/tmp/log.sock", "test",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["logger", "--bogus"]));
    }
}

mod look {
    use super::parse;
    use linuxutils_misc::look::Args;

    // ---- SYNOPSIS form: look [options] <string> [<file>]

    #[test]
    fn string_only() {
        insta::assert_snapshot!(parse::<Args>(&["look", "abc"]));
    }

    #[test]
    fn string_and_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "look", "abc", "/usr/share/dict/words",
        ]));
    }

    #[test]
    fn alternative_dict() {
        insta::assert_snapshot!(parse::<Args>(&["look", "-a", "abc"]));
    }

    #[test]
    fn ignore_case_alphanum() {
        insta::assert_snapshot!(parse::<Args>(&[
            "look", "-d", "-f", "abc",
        ]));
    }

    #[test]
    fn terminate_char() {
        insta::assert_snapshot!(parse::<Args>(&[
            "look", "-t", "/", "abc",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_string() {
        insta::assert_snapshot!(parse::<Args>(&["look"]));
    }

    #[test]
    fn err_terminate_multichar() {
        insta::assert_snapshot!(parse::<Args>(&[
            "look", "-t", "abc", "needle",
        ]));
    }
}

mod lslocks {
    use super::parse;
    use linuxutils_misc::lslocks::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["lslocks"]));
    }

    #[test]
    fn pid_filter() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lslocks", "-p", "1234",
        ]));
    }

    #[test]
    fn json_output() {
        insta::assert_snapshot!(parse::<Args>(&["lslocks", "-J"]));
    }

    #[test]
    fn output_columns() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lslocks", "-o", "PID,COMMAND,TYPE,SIZE",
        ]));
    }

    #[test]
    fn output_all_and_raw() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lslocks", "--output-all", "-r",
        ]));
    }

    #[test]
    fn bytes_noheadings_notruncate() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lslocks", "-b", "-n", "-u",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_pid_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lslocks", "-p", "abc",
        ]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["lslocks", "--bogus"]));
    }
}

mod mcookie {
    use super::parse;
    use linuxutils_misc::mcookie::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["mcookie"]));
    }

    #[test]
    fn from_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mcookie", "-f", "/dev/urandom",
        ]));
    }

    #[test]
    fn from_stdin() {
        insta::assert_snapshot!(parse::<Args>(&["mcookie", "-f", "-"]));
    }

    #[test]
    fn max_size_and_verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mcookie", "-f", "/tmp/seed", "-m", "1024", "-v",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_max_size_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mcookie", "-m", "abc",
        ]));
    }
}

mod namei {
    use super::parse;
    use linuxutils_misc::namei::Args;

    #[test]
    fn single_path() {
        insta::assert_snapshot!(parse::<Args>(&[
            "namei", "/usr/bin/ls",
        ]));
    }

    #[test]
    fn multiple_paths() {
        insta::assert_snapshot!(parse::<Args>(&[
            "namei", "/etc/passwd", "/var/log/messages",
        ]));
    }

    #[test]
    fn long_listing() {
        insta::assert_snapshot!(parse::<Args>(&["namei", "-l", "/etc"]));
    }

    #[test]
    fn modes_owners_vertical() {
        insta::assert_snapshot!(parse::<Args>(&[
            "namei", "-m", "-o", "-v", "/etc",
        ]));
    }

    #[test]
    fn nosymlinks_mountpoints() {
        insta::assert_snapshot!(parse::<Args>(&[
            "namei", "-n", "-x", "/var",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_paths() {
        insta::assert_snapshot!(parse::<Args>(&["namei"]));
    }
}

mod rename {
    use super::parse;
    use linuxutils_misc::rename::Args;

    // ---- SYNOPSIS form: rename [options] <expression> <replacement> <file>...

    #[test]
    fn basic_rename() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rename", ".txt", ".md", "a.txt", "b.txt",
        ]));
    }

    #[test]
    fn verbose_no_act() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rename", "-v", "-n", ".bak", "", "old.bak",
        ]));
    }

    #[test]
    fn all_occurrences() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rename", "-a", "foo", "bar", "foofoo.txt",
        ]));
    }

    #[test]
    fn last_occurrence() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rename", "-l", "foo", "bar", "foofoo.txt",
        ]));
    }

    #[test]
    fn no_overwrite_interactive() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rename", "-o", "-i", "x", "y", "x.txt",
        ]));
    }

    #[test]
    fn symlink_mode() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rename", "-s", "old", "new", "link.old",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_replacement() {
        insta::assert_snapshot!(parse::<Args>(&["rename", "expr"]));
    }

    #[test]
    fn err_missing_files() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rename", "expr", "repl",
        ]));
    }
}

mod uuidgen {
    use super::parse;
    use linuxutils_misc::uuidgen::Args;

    #[test]
    fn no_args_default_v4() {
        insta::assert_snapshot!(parse::<Args>(&["uuidgen"]));
    }

    #[test]
    fn random_v4() {
        insta::assert_snapshot!(parse::<Args>(&["uuidgen", "-r"]));
    }

    #[test]
    fn time_v1() {
        insta::assert_snapshot!(parse::<Args>(&["uuidgen", "-t"]));
    }

    #[test]
    fn time_v6() {
        insta::assert_snapshot!(parse::<Args>(&["uuidgen", "-6"]));
    }

    #[test]
    fn time_v7() {
        insta::assert_snapshot!(parse::<Args>(&["uuidgen", "-7"]));
    }

    #[test]
    fn md5_with_namespace_and_name() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidgen",
            "--md5",
            "-n",
            "@dns",
            "-N",
            "example.com",
        ]));
    }

    #[test]
    fn sha1_with_namespace_and_name() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidgen",
            "--sha1",
            "-n",
            "@url",
            "-N",
            "https://example.com",
        ]));
    }

    #[test]
    fn count_multiple() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidgen", "-r", "-C", "5",
        ]));
    }

    #[test]
    fn hex_name() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidgen",
            "--md5",
            "-n",
            "@dns",
            "-N",
            "deadbeef",
            "-x",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_md5_without_namespace() {
        insta::assert_snapshot!(parse::<Args>(&["uuidgen", "--md5"]));
    }

    #[test]
    fn err_count_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidgen", "-C", "abc",
        ]));
    }
}

mod uuidparse {
    use super::parse;
    use linuxutils_misc::uuidparse::Args;

    #[test]
    fn single_uuid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidparse",
            "00000000-0000-0000-0000-000000000000",
        ]));
    }

    #[test]
    fn multiple_uuids() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidparse",
            "00000000-0000-0000-0000-000000000000",
            "ffffffff-ffff-ffff-ffff-ffffffffffff",
        ]));
    }

    #[test]
    fn json_output() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidparse",
            "-J",
            "00000000-0000-0000-0000-000000000000",
        ]));
    }

    #[test]
    fn raw_noheadings() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidparse",
            "-r",
            "-n",
            "00000000-0000-0000-0000-000000000000",
        ]));
    }

    #[test]
    fn output_columns() {
        insta::assert_snapshot!(parse::<Args>(&[
            "uuidparse",
            "-o",
            "UUID,VARIANT,TYPE",
            "00000000-0000-0000-0000-000000000000",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_uuids() {
        insta::assert_snapshot!(parse::<Args>(&["uuidparse"]));
    }
}

mod waitpid {
    use super::parse;
    use linuxutils_misc::waitpid::Args;

    #[test]
    fn single_pid() {
        insta::assert_snapshot!(parse::<Args>(&["waitpid", "1234"]));
    }

    #[test]
    fn multiple_pids() {
        insta::assert_snapshot!(parse::<Args>(&[
            "waitpid", "1234", "5678",
        ]));
    }

    #[test]
    fn timeout_and_verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "waitpid", "-v", "-t", "30", "1234",
        ]));
    }

    #[test]
    fn count_and_exited_ok() {
        insta::assert_snapshot!(parse::<Args>(&[
            "waitpid", "-e", "-c", "2", "1234", "5678", "9999",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_pids() {
        insta::assert_snapshot!(parse::<Args>(&["waitpid"]));
    }

    #[test]
    fn err_pid_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["waitpid", "abc"]));
    }
}

mod whereis {
    // whereis hand-rolls its own parser (not clap), so we can't snapshot
    // a Result<Args, Error>. Instead we snapshot the parsed `Args` Debug
    // directly — it implements Debug + Default, and `parse_from(&[..])`
    // is infallible (it std::process::exit(1) on syntax errors, which
    // can't easily be tested in-process).

    use linuxutils_misc::whereis::Args;

    fn parse(argv: &[&str]) -> String {
        let args: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
        format!("{:#?}", Args::parse_from(&args))
    }

    #[test]
    fn name_only() {
        insta::assert_snapshot!(parse(&["ls"]));
    }

    #[test]
    fn binaries_only() {
        insta::assert_snapshot!(parse(&["-b", "ls"]));
    }

    #[test]
    fn manuals_and_sources() {
        insta::assert_snapshot!(parse(&["-m", "-s", "ls"]));
    }

    #[test]
    fn unusual_only() {
        insta::assert_snapshot!(parse(&["-u", "ls"]));
    }

    #[test]
    fn list_paths() {
        insta::assert_snapshot!(parse(&["-l"]));
    }

    #[test]
    fn glob_mode() {
        insta::assert_snapshot!(parse(&["-g", "ls*"]));
    }

    #[test]
    fn combined_short_flags() {
        insta::assert_snapshot!(parse(&["-bm", "ls"]));
    }

    #[test]
    fn custom_bin_dirs() {
        insta::assert_snapshot!(parse(&[
            "-B", "/opt/bin", "/usr/local/bin", "-f", "ls",
        ]));
    }

    #[test]
    fn custom_man_and_src_dirs() {
        insta::assert_snapshot!(parse(&[
            "-M",
            "/usr/share/man",
            "-f",
            "-S",
            "/usr/src",
            "-f",
            "ls",
        ]));
    }

    #[test]
    fn multiple_names() {
        insta::assert_snapshot!(parse(&["ls", "cat", "grep"]));
    }
}
