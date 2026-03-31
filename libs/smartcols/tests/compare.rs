use similar::TextDiff;
use std::{env, process::Command};

fn run_system(command: &str, args: &[&str]) -> std::process::Output {
    Command::new(command)
        .args(args)
        .output()
        .expect("can run system command")
}

fn run_ours(command: &str, args: &[&str]) -> std::process::Output {
    Command::new(command)
        .args(args)
        .env("LD_PRELOAD", library_path())
        .output()
        .expect("can run our command")
}

fn library_path() -> String {
    format!(
        "{}/../../target/debug/libsmartcols.so",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    )
}

fn print_unified(old: &str, new: &str) {
    let diff = TextDiff::from_lines(old, new);
    println!("{}", diff.unified_diff().header("system", "ours"));
}

fn assert_output_eq(
    system: &std::process::Output,
    ours: &std::process::Output,
    label: &str,
) {
    assert_eq!(
        system.status, ours.status,
        "exit status mismatch for: {label}"
    );

    let sys_out = String::from_utf8_lossy(&system.stdout);
    let our_out = String::from_utf8_lossy(&ours.stdout);

    print_unified(&sys_out, &our_out);
    print_unified(
        &String::from_utf8_lossy(&system.stderr),
        &String::from_utf8_lossy(&ours.stderr),
    );

    assert_eq!(sys_out, our_out, "stdout mismatch for: {label}");
}

fn assert_json_eq(
    system: &std::process::Output,
    ours: &std::process::Output,
    label: &str,
) {
    assert_eq!(
        system.status, ours.status,
        "exit status mismatch for: {label}"
    );

    let sys_out = String::from_utf8_lossy(&system.stdout);
    let our_out = String::from_utf8_lossy(&ours.stdout);

    let sys_json: serde_json::Value = serde_json::from_str(&sys_out)
        .expect("system output is not valid JSON");
    let our_json: serde_json::Value =
        serde_json::from_str(&our_out).expect("our output is not valid JSON");

    if sys_json != our_json {
        print_unified(
            &serde_json::to_string_pretty(&sys_json).unwrap(),
            &serde_json::to_string_pretty(&our_json).unwrap(),
        );
        panic!("JSON data mismatch for: {label}");
    }
}

fn compare_output(command: &str, args: &[&str]) {
    let system = run_system(command, args);
    let ours = run_ours(command, args);
    let label = format!("{command} {}", args.join(" "));
    assert_output_eq(&system, &ours, &label);
}

fn compare_json_output(command: &str, args: &[&str]) {
    let system = run_system(command, args);
    let ours = run_ours(command, args);
    let label = format!("{command} {}", args.join(" "));
    assert_json_eq(&system, &ours, &label);
}

fn run_with_stdin(
    command: &str,
    args: &[&str],
    stdin: &[u8],
    preload: bool,
) -> std::process::Output {
    use std::{io::Write, process::Stdio};

    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if preload {
        cmd.env("LD_PRELOAD", library_path());
    }
    let mut child = cmd.spawn().expect("can spawn command");
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

fn compare_output_stdin(command: &str, args: &[&str], stdin: &[u8]) {
    let system = run_with_stdin(command, args, stdin, false);
    let ours = run_with_stdin(command, args, stdin, true);
    let label = format!("{command} {}", args.join(" "));
    assert_output_eq(&system, &ours, &label);
}

fn compare_json_output_stdin(command: &str, args: &[&str], stdin: &[u8]) {
    let system = run_with_stdin(command, args, stdin, false);
    let ours = run_with_stdin(command, args, stdin, true);
    let label = format!("{command} {}", args.join(" "));
    assert_json_eq(&system, &ours, &label);
}

// ---- lsblk ----------------------------------------------------------------

#[test]
#[ignore = "needs data_func + wrapzero support"]
fn compare_lsblk() {
    compare_output("lsblk", &[]);
}

#[test]
#[ignore = "needs data_func + wrapzero support"]
fn compare_lsblk_raw() {
    compare_output("lsblk", &["--raw"]);
}

#[test]
#[ignore = "needs data_func + wrapzero support"]
fn compare_lsblk_json() {
    compare_json_output("lsblk", &["--json"]);
}

#[test]
#[ignore = "needs data_func + wrapzero support"]
fn compare_lsblk_pairs() {
    compare_output("lsblk", &["--pairs"]);
}

#[test]
#[ignore = "needs data_func + wrapzero support"]
fn compare_lsblk_list() {
    compare_output("lsblk", &["--list"]);
}

// ---- lsns ------------------------------------------------------------------

#[test]
#[ignore]
fn compare_lsns() {
    compare_output("lsns", &[]);
}

#[test]
#[ignore]
fn compare_lsns_raw() {
    compare_output("lsns", &["--raw"]);
}

#[test]
#[ignore]
fn compare_lsns_json() {
    compare_json_output("lsns", &["--json"]);
}

#[test]
#[ignore]
fn compare_lsns_noheadings() {
    compare_output("lsns", &["--noheadings"]);
}

// ---- lsipc -----------------------------------------------------------------

#[test]
#[ignore]
fn compare_lsipc() {
    compare_output("lsipc", &[]);
}

#[test]
#[ignore]
fn compare_lsipc_raw() {
    compare_output("lsipc", &["--raw"]);
}

#[test]
#[ignore]
fn compare_lsipc_json() {
    compare_json_output("lsipc", &["--json"]);
}

#[test]
#[ignore]
fn compare_lsipc_export() {
    compare_output("lsipc", &["--export"]);
}

// ---- lsmem -----------------------------------------------------------------

#[test]
#[ignore]
fn compare_lsmem() {
    compare_output("lsmem", &[]);
}

#[test]
#[ignore]
fn compare_lsmem_raw() {
    compare_output("lsmem", &["--raw"]);
}

#[test]
#[ignore]
fn compare_lsmem_json() {
    compare_json_output("lsmem", &["--json"]);
}

#[test]
#[ignore]
fn compare_lsmem_pairs() {
    compare_output("lsmem", &["--pairs"]);
}

#[test]
#[ignore]
fn compare_lsmem_noheadings() {
    compare_output("lsmem", &["--noheadings"]);
}

// ---- lslogins --------------------------------------------------------------

#[test]
#[ignore]
fn compare_lslogins() {
    compare_output("lslogins", &[]);
}

#[test]
#[ignore]
fn compare_lslogins_raw() {
    compare_output("lslogins", &["--raw"]);
}

#[test]
#[ignore]
fn compare_lslogins_json() {
    compare_output("lslogins", &["--json"]);
}

#[test]
#[ignore]
fn compare_lslogins_export() {
    compare_output("lslogins", &["--export"]);
}

// ---- lsirq -----------------------------------------------------------------

#[test]
#[ignore]
fn compare_lsirq() {
    compare_output("lsirq", &[]);
}

#[test]
#[ignore]
fn compare_lsirq_raw() {
    compare_output("lsirq", &["--raw"]);
}

#[test]
#[ignore]
fn compare_lsirq_json() {
    compare_json_output("lsirq", &["--json"]);
}

#[test]
#[ignore]
fn compare_lsirq_pairs() {
    compare_output("lsirq", &["--pairs"]);
}

// ---- findmnt ---------------------------------------------------------------

#[test]
#[ignore]
fn compare_findmnt() {
    compare_output("findmnt", &[]);
}

#[test]
#[ignore]
fn compare_findmnt_raw() {
    compare_output("findmnt", &["--raw"]);
}

#[test]
#[ignore]
fn compare_findmnt_json() {
    compare_json_output("findmnt", &["--json"]);
}

#[test]
#[ignore]
fn compare_findmnt_list() {
    compare_output("findmnt", &["--list"]);
}

#[test]
#[ignore]
fn compare_findmnt_pairs() {
    compare_output("findmnt", &["--pairs"]);
}

// ---- prlimit ---------------------------------------------------------------

#[test]
#[ignore]
fn compare_prlimit() {
    compare_output("prlimit", &[]);
}

#[test]
#[ignore]
fn compare_prlimit_raw() {
    compare_output("prlimit", &["--raw"]);
}

#[test]
#[ignore]
fn compare_prlimit_noheadings() {
    compare_output("prlimit", &["--noheadings"]);
}

// ---- rfkill ----------------------------------------------------------------

#[test]
#[ignore]
fn compare_rfkill() {
    compare_output("rfkill", &[]);
}

#[test]
#[ignore]
fn compare_rfkill_raw() {
    compare_output("rfkill", &["--raw"]);
}

#[test]
#[ignore]
fn compare_rfkill_json() {
    compare_json_output("rfkill", &["--json"]);
}

#[test]
#[ignore]
fn compare_rfkill_noheadings() {
    compare_output("rfkill", &["--noheadings"]);
}

// ---- lscpu -----------------------------------------------------------------

#[test]
#[ignore]
fn compare_lscpu() {
    compare_output("lscpu", &[]);
}

#[test]
#[ignore]
fn compare_lscpu_json() {
    compare_json_output("lscpu", &["--json"]);
}

// ---- swapon ----------------------------------------------------------------

#[test]
#[ignore]
fn compare_swapon_show() {
    compare_output("swapon", &["--show"]);
}

#[test]
#[ignore]
fn compare_swapon_show_raw() {
    compare_output("swapon", &["--show", "--raw"]);
}

#[test]
#[ignore]
fn compare_swapon_show_noheadings() {
    compare_output("swapon", &["--show", "--noheadings"]);
}

// ---- losetup ---------------------------------------------------------------

#[test]
#[ignore]
fn compare_losetup_list() {
    compare_output("losetup", &["--list"]);
}

#[test]
#[ignore]
fn compare_losetup_list_raw() {
    compare_output("losetup", &["--list", "--raw"]);
}

#[test]
#[ignore]
fn compare_losetup_list_json() {
    compare_json_output("losetup", &["--list", "--json"]);
}

// ---- lslocks ---------------------------------------------------------------

#[test]
#[ignore]
fn compare_lslocks() {
    compare_output("lslocks", &[]);
}

#[test]
#[ignore]
fn compare_lslocks_raw() {
    compare_output("lslocks", &["--raw"]);
}

#[test]
#[ignore]
fn compare_lslocks_json() {
    compare_json_output("lslocks", &["--json"]);
}

#[test]
#[ignore]
fn compare_lslocks_noheadings() {
    compare_output("lslocks", &["--noheadings"]);
}

// ---- fincore ---------------------------------------------------------------

#[test]
#[ignore]
fn compare_fincore() {
    compare_output("fincore", &["/usr/bin/ls"]);
}

#[test]
#[ignore]
fn compare_fincore_raw() {
    compare_output("fincore", &["--raw", "/usr/bin/ls"]);
}

#[test]
#[ignore]
fn compare_fincore_json() {
    compare_json_output("fincore", &["--json", "/usr/bin/ls"]);
}

// ---- wdctl -----------------------------------------------------------------

#[test]
#[ignore]
fn compare_wdctl() {
    compare_output("wdctl", &[]);
}

#[test]
#[ignore]
fn compare_wdctl_raw() {
    compare_output("wdctl", &["--raw"]);
}

// ---- column ----------------------------------------------------------------

const COLUMN_INPUT: &[u8] = b"Name  Age  City
Alice  30  New York
Bob  25  London
Charlie  35  Paris
";

const COLUMN_INPUT_LONG: &[u8] = b"PID  USER  COMMAND  STATUS
1  root  /sbin/init  sleeping
432  www-data  /usr/sbin/apache2 -k start  sleeping
1523  postgres  postgres: checkpointer  idle
8821  patrick  /usr/lib64/firefox/firefox  running
9102  patrick  /usr/bin/gnome-shell  sleeping
";

#[test]
#[ignore]
fn compare_column_table() {
    compare_output_stdin("column", &["--table"], COLUMN_INPUT);
}

#[test]
#[ignore]
fn compare_column_table_noheadings() {
    compare_output_stdin(
        "column",
        &["--table", "--table-noheadings"],
        COLUMN_INPUT,
    );
}

#[test]
#[ignore]
fn compare_column_json() {
    compare_output_stdin(
        "column",
        &["--table", "--json", "--table-name", "people"],
        COLUMN_INPUT,
    );
}

#[test]
#[ignore]
fn compare_column_table_right() {
    compare_output_stdin(
        "column",
        &["--table", "--table-right", "1,2"],
        COLUMN_INPUT,
    );
}

#[test]
#[ignore]
fn compare_column_table_long() {
    compare_output_stdin("column", &["--table"], COLUMN_INPUT_LONG);
}

#[test]
#[ignore]
fn compare_column_json_long() {
    compare_output_stdin(
        "column",
        &["--table", "--json", "--table-name", "processes"],
        COLUMN_INPUT_LONG,
    );
}

#[test]
#[ignore]
fn compare_column_separator() {
    compare_output_stdin(
        "column",
        &["--table", "--separator", ","],
        b"a,b,c\n1,2,3\n4,5,6\n",
    );
}

#[test]
#[ignore]
fn compare_column_output_separator() {
    compare_output_stdin(
        "column",
        &["--table", "--output-separator", " | "],
        COLUMN_INPUT,
    );
}

// ---- lspci (pciutils, not util-linux but uses libsmartcols) ----------------

#[test]
#[ignore]
fn compare_lspci() {
    compare_output("lspci", &[]);
}

#[test]
#[ignore]
fn compare_lspci_k() {
    compare_output("lspci", &["-k"]);
}
