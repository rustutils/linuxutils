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

mod bits {
    use super::parse;
    use linuxutils_text::bits::Args;

    // ---- SYNOPSIS form: bits [-w N] [<MODE>] [<MASK_OR_LIST>...]

    #[test]
    fn default_mode() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "0xeec2",
        ]));
    }

    #[test]
    fn list_mode_multiple_groups() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "--list", "4,5-8", "16,30",
        ]));
    }

    #[test]
    fn binary_mode() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "-b", "0x10",
        ]));
    }

    #[test]
    fn grouped_mask_mode() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "--grouped-mask", "2,22,74,79",
        ]));
    }

    #[test]
    fn explicit_mask_mode() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "-m", "0x1",
        ]));
    }

    #[test]
    fn width_override() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "-w", "64", "--list", "2,22,74,79",
        ]));
    }

    #[test]
    fn no_args_reads_stdin() {
        insta::assert_snapshot!(parse::<Args>(&["bits"]));
    }

    // ---- Error cases

    #[test]
    fn err_conflicting_modes() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "-l", "-b", "0x1",
        ]));
    }

    #[test]
    fn err_width_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "bits", "-w", "abc",
        ]));
    }
}

mod colrm {
    use super::parse;
    use linuxutils_text::colrm::Args;

    // ---- SYNOPSIS form: colrm [first [last]]

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["colrm"]));
    }

    #[test]
    fn first_only() {
        insta::assert_snapshot!(parse::<Args>(&["colrm", "10"]));
    }

    #[test]
    fn first_and_last() {
        insta::assert_snapshot!(parse::<Args>(&["colrm", "10", "20"]));
    }

    // ---- Error cases

    #[test]
    fn err_first_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["colrm", "abc"]));
    }

    #[test]
    fn err_too_many_args() {
        insta::assert_snapshot!(parse::<Args>(&[
            "colrm", "1", "2", "3",
        ]));
    }
}

mod column {
    use super::parse;
    use linuxutils_text::column::Args;

    // ---- Stream mode (no -t)

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["column"]));
    }

    #[test]
    fn input_files() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column", "a.txt", "b.txt",
        ]));
    }

    #[test]
    fn fillrows() {
        insta::assert_snapshot!(parse::<Args>(&["column", "-x"]));
    }

    #[test]
    fn output_width() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column", "-c", "120",
        ]));
    }

    // ---- Table mode (-t)

    #[test]
    fn table_basic() {
        insta::assert_snapshot!(parse::<Args>(&["column", "-t"]));
    }

    #[test]
    fn table_with_columns() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column", "-t", "-N", "NAME,SIZE,USE",
        ]));
    }

    #[test]
    fn table_with_separator() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column", "-t", "-s", ":",
        ]));
    }

    #[test]
    fn table_noheadings_and_output_separator() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column", "-t", "-d", "-o", " | ",
        ]));
    }

    #[test]
    fn json_output() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column",
            "-J",
            "-N",
            "NAME,SIZE",
            "-n",
            "items",
        ]));
    }

    #[test]
    fn tree_view() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column",
            "-t",
            "-N",
            "NAME,PID,PPID",
            "-r",
            "NAME",
            "-i",
            "PID",
            "-p",
            "PPID",
        ]));
    }

    #[test]
    fn repeated_table_column() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column",
            "-t",
            "-C",
            "name=NAME,trunc",
            "-C",
            "name=SIZE,right",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["column", "--bogus"]));
    }

    #[test]
    fn err_table_columns_limit_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "column", "-t", "-l", "abc",
        ]));
    }
}

mod hexdump {
    use super::parse;
    use linuxutils_text::hexdump::Args;

    // ---- Format flags

    #[test]
    fn canonical() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-C", "input.bin",
        ]));
    }

    #[test]
    fn one_byte_octal() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-b", "input.bin",
        ]));
    }

    #[test]
    fn one_byte_hex() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-X", "input.bin",
        ]));
    }

    #[test]
    fn two_bytes_decimal() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-d", "input.bin",
        ]));
    }

    // ---- Length and skip

    #[test]
    fn length_and_skip() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-n", "256", "-s", "1024", "input.bin",
        ]));
    }

    #[test]
    fn no_squeezing() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-v", "-C", "input.bin",
        ]));
    }

    #[test]
    fn multiple_files() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-C", "a.bin", "b.bin",
        ]));
    }

    #[test]
    fn no_args_reads_stdin() {
        insta::assert_snapshot!(parse::<Args>(&["hexdump"]));
    }

    // ---- Error cases

    #[test]
    fn err_length_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "hexdump", "-n", "abc",
        ]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["hexdump", "--bogus"]));
    }
}

mod line {
    use super::parse;
    use linuxutils_text::line::Args;

    // line takes no flags or positionals — only the conventional --help / --version.

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["line"]));
    }

    #[test]
    fn err_unexpected_positional() {
        insta::assert_snapshot!(parse::<Args>(&["line", "extra"]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["line", "--bogus"]));
    }
}

mod rev {
    use super::parse;
    use linuxutils_text::rev::Args;

    // ---- SYNOPSIS form: rev [options] [file...]

    #[test]
    fn no_args_reads_stdin() {
        insta::assert_snapshot!(parse::<Args>(&["rev"]));
    }

    #[test]
    fn single_file() {
        insta::assert_snapshot!(parse::<Args>(&["rev", "input.txt"]));
    }

    #[test]
    fn multiple_files() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rev", "a.txt", "b.txt",
        ]));
    }

    #[test]
    fn zero_separator_short() {
        insta::assert_snapshot!(parse::<Args>(&["rev", "-0"]));
    }

    #[test]
    fn zero_separator_long() {
        insta::assert_snapshot!(parse::<Args>(&["rev", "--zero"]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["rev", "--bogus"]));
    }
}
