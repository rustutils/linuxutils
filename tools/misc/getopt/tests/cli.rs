use assert_cmd::Command;
use predicates::prelude::*;

fn getopt() -> Command {
    Command::cargo_bin("getopt").unwrap()
}

// --- Enhanced getopt test (-T) ---

#[test]
fn test_flag_returns_exit_4() {
    getopt().arg("-T").assert().code(4);
}

// --- Basic short options ---

#[test]
fn short_option_no_arg() {
    getopt()
        .args(["-o", "abc", "--", "-a", "-b", "-c"])
        .assert()
        .success()
        .stdout(" -a -b -c --\n");
}

#[test]
fn short_option_required_arg_separate() {
    getopt()
        .args(["-o", "ab:c", "--", "-b", "value"])
        .assert()
        .success()
        .stdout(" -b 'value' --\n");
}

#[test]
fn short_option_required_arg_attached() {
    getopt()
        .args(["-o", "ab:c", "--", "-bvalue"])
        .assert()
        .success()
        .stdout(" -b 'value' --\n");
}

#[test]
fn short_option_optional_arg_attached() {
    getopt()
        .args(["-o", "ab::c", "--", "-bval"])
        .assert()
        .success()
        .stdout(" -b 'val' --\n");
}

#[test]
fn short_option_optional_arg_missing() {
    getopt()
        .args(["-o", "ab::c", "--", "-b", "-c"])
        .assert()
        .success()
        .stdout(" -b '' -c --\n");
}

#[test]
fn combined_short_options() {
    getopt()
        .args(["-o", "abc", "--", "-abc"])
        .assert()
        .success()
        .stdout(" -a -b -c --\n");
}

#[test]
fn combined_short_with_arg() {
    getopt()
        .args(["-o", "ab:c", "--", "-ab", "val", "-c"])
        .assert()
        .success()
        .stdout(" -a -b 'val' -c --\n");
}

// --- Long options ---

#[test]
fn long_option_no_arg() {
    getopt()
        .args(["-o", "", "-l", "verbose", "--", "--verbose"])
        .assert()
        .success()
        .stdout(" --verbose --\n");
}

#[test]
fn long_option_required_arg_equals() {
    getopt()
        .args(["-o", "", "-l", "output:", "--", "--output=file.txt"])
        .assert()
        .success()
        .stdout(" --output 'file.txt' --\n");
}

#[test]
fn long_option_required_arg_separate() {
    getopt()
        .args(["-o", "", "-l", "output:", "--", "--output", "file.txt"])
        .assert()
        .success()
        .stdout(" --output 'file.txt' --\n");
}

#[test]
fn long_option_optional_arg_equals() {
    getopt()
        .args(["-o", "", "-l", "debug::", "--", "--debug=3"])
        .assert()
        .success()
        .stdout(" --debug '3' --\n");
}

#[test]
fn long_option_optional_arg_missing() {
    getopt()
        .args(["-o", "", "-l", "debug::", "--", "--debug"])
        .assert()
        .success()
        .stdout(" --debug '' --\n");
}

#[test]
fn long_option_abbreviation() {
    getopt()
        .args(["-o", "", "-l", "verbose,version", "--", "--verbo"])
        .assert()
        .success()
        .stdout(" --verbose --\n");
}

#[test]
fn long_option_ambiguous_abbreviation() {
    getopt()
        .args(["-o", "", "-l", "verbose,version", "--", "--ver"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ambiguous"));
}

#[test]
fn multiple_longoptions_flags() {
    getopt()
        .args([
            "-o", "a", "-l", "foo", "-l", "bar:", "--", "-a", "--foo", "--bar",
            "val",
        ])
        .assert()
        .success()
        .stdout(" -a --foo --bar 'val' --\n");
}

// --- Mixed short and long ---

#[test]
fn mixed_short_and_long() {
    getopt()
        .args([
            "-o",
            "ab:c",
            "-l",
            "long1,long2:",
            "--",
            "-a",
            "--long1",
            "--long2=value",
            "-c",
            "extra",
        ])
        .assert()
        .success()
        .stdout(" -a --long1 --long2 'value' -c -- 'extra'\n");
}

// --- Non-option parameters ---

#[test]
fn non_option_collected_at_end() {
    getopt()
        .args(["-o", "ab:c", "--", "-a", "extra1", "-b", "val", "extra2"])
        .assert()
        .success()
        .stdout(" -a -b 'val' -- 'extra1' 'extra2'\n");
}

#[test]
fn double_dash_stops_parsing() {
    getopt()
        .args(["-o", "ab:c", "--", "-a", "--", "-b", "val"])
        .assert()
        .success()
        .stdout(" -a -- '-b' 'val'\n");
}

// --- Scanning modes ---

#[test]
fn stop_at_first_nonopt_plus_prefix() {
    getopt()
        .args(["-o", "+ab:c", "--", "-a", "extra", "-b", "val", "-c"])
        .assert()
        .success()
        .stdout(" -a -- 'extra' '-b' 'val' '-c'\n");
}

#[test]
fn in_place_mode_dash_prefix() {
    getopt()
        .args(["-o", "-ab:c", "--", "-a", "extra", "-b", "val", "-c"])
        .assert()
        .success()
        .stdout(" -a 'extra' -b 'val' -c --\n");
}

// --- Error handling ---

#[test]
fn unknown_short_option() {
    getopt()
        .args(["-o", "abc", "--", "-a", "-x", "-c"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid option -- 'x'"))
        .stdout(" -a -c --\n");
}

#[test]
fn missing_required_arg_short() {
    getopt()
        .args(["-o", "ab:", "--", "-a", "-b"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "option requires an argument -- 'b'",
        ));
}

#[test]
fn unknown_long_option() {
    getopt()
        .args(["-o", "", "-l", "verbose", "--", "--unknown"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized option '--unknown'"));
}

#[test]
fn missing_required_arg_long() {
    getopt()
        .args(["-o", "", "-l", "output:", "--", "--output"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires an argument"));
}

// --- Program name in errors (-n) ---

#[test]
fn custom_prog_name_in_errors() {
    getopt()
        .args(["-o", "a", "-n", "myprog", "--", "-x"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("myprog:"));
}

// --- Quiet mode (-q) ---

#[test]
fn quiet_suppresses_errors() {
    getopt()
        .args(["-o", "a", "-q", "--", "-x"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty());
}

// --- Quiet output (-Q) ---

#[test]
fn quiet_output_suppresses_stdout() {
    getopt()
        .args(["-o", "abc", "-Q", "--", "-a", "-b", "-c"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn quiet_output_still_shows_errors() {
    getopt()
        .args(["-o", "a", "-Q", "--", "-x"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid option"));
}

// --- Unquoted mode (-u) ---

#[test]
fn unquoted_output() {
    getopt()
        .args(["-o", "ab:c", "-u", "--", "-a", "-b", "value", "-c", "extra"])
        .assert()
        .success()
        .stdout(" -a -b value -c -- extra\n");
}

// --- Alternative mode (-a) ---

#[test]
fn alternative_single_dash_long() {
    getopt()
        .args([
            "-o",
            "a",
            "-a",
            "-l",
            "verbose,output:",
            "--",
            "-verbose",
            "-output=file",
        ])
        .assert()
        .success()
        .stdout(" --verbose --output 'file' --\n");
}

#[test]
fn alternative_still_allows_double_dash() {
    getopt()
        .args(["-o", "", "-a", "-l", "verbose", "--", "--verbose"])
        .assert()
        .success()
        .stdout(" --verbose --\n");
}

// --- Quoting ---

#[test]
fn quoting_with_spaces() {
    getopt()
        .args(["-o", "b:", "--", "-b", "hello world"])
        .assert()
        .success()
        .stdout(" -b 'hello world' --\n");
}

#[test]
fn quoting_with_single_quotes() {
    getopt()
        .args(["-o", "b:", "--", "-b", "it's"])
        .assert()
        .success()
        .stdout(" -b 'it'\\''s' --\n");
}

#[test]
fn quoting_empty_string() {
    getopt()
        .args(["-o", "b::", "--", "-b"])
        .assert()
        .success()
        .stdout(" -b '' --\n");
}

#[test]
fn csh_quoting() {
    getopt()
        .args(["-o", "b:", "-s", "csh", "--", "-b", "hello world"])
        .assert()
        .success()
        .stdout(" -b 'hello world' --\n");
}

// --- Compat mode (no -o, first param is optstring) ---

#[test]
fn compat_mode_first_param_is_optstring() {
    getopt()
        .args(["ab:c", "-a", "-b", "val", "-c", "extra"])
        .assert()
        .success()
        .stdout(" -a -b 'val' -c -- 'extra'\n");
}

// --- Edge cases ---

#[test]
fn no_options_only_nonopts() {
    getopt()
        .args(["-o", "", "--", "foo", "bar"])
        .assert()
        .success()
        .stdout(" -- 'foo' 'bar'\n");
}

#[test]
fn empty_input() {
    getopt()
        .args(["-o", "abc", "--"])
        .assert()
        .success()
        .stdout(" --\n");
}

#[test]
fn missing_optstring_no_params() {
    getopt()
        .assert()
        .code(2)
        .stderr(predicate::str::contains("missing optstring"));
}

#[test]
fn unknown_shell() {
    getopt()
        .args(["-o", "a", "-s", "fish", "--", "-a"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unknown shell"));
}

#[test]
fn leading_colon_in_optstring() {
    // Leading : is accepted (suppresses errors in traditional getopt, we handle via -q)
    getopt()
        .args(["-o", ":ab:c", "--", "-a", "-b", "val"])
        .assert()
        .success()
        .stdout(" -a -b 'val' --\n");
}

#[test]
fn only_double_dash_in_user_args() {
    getopt()
        .args(["-o", "abc", "--", "--"])
        .assert()
        .success()
        .stdout(" --\n");
}

#[test]
fn non_option_with_special_chars() {
    getopt()
        .args(["-o", "a", "--", "-a", "hello world", "it's"])
        .assert()
        .success()
        .stdout(" -a -- 'hello world' 'it'\\''s'\n");
}
