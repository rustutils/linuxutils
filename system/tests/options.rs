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

mod blkdiscard {
    use super::parse;
    use linuxutils_system::blkdiscard::Args;

    // ---- SYNOPSIS form: blkdiscard [options] <device>

    #[test]
    fn device_only() {
        insta::assert_snapshot!(parse::<Args>(&["blkdiscard", "/dev/sda"]));
    }

    #[test]
    fn offset_and_length() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blkdiscard",
            "-o",
            "1048576",
            "-l",
            "10485760",
            "/dev/sda",
        ]));
    }

    #[test]
    fn secure_discard() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blkdiscard",
            "-s",
            "/dev/sda",
        ]));
    }

    #[test]
    fn zeroout_with_step() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blkdiscard",
            "-z",
            "-p",
            "1048576",
            "/dev/sda",
        ]));
    }

    #[test]
    fn force_quiet_verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blkdiscard",
            "-f",
            "-q",
            "-v",
            "/dev/sda",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_device() {
        insta::assert_snapshot!(parse::<Args>(&["blkdiscard"]));
    }

    #[test]
    fn err_offset_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blkdiscard",
            "-o",
            "abc",
            "/dev/sda",
        ]));
    }
}

mod choom {
    use super::parse;
    use linuxutils_system::choom::Args;

    // choom uses `command: Vec<String>` with `trailing_var_arg = true,
    // allow_hyphen_values = true`, so unknown flags after a command get
    // consumed as positionals rather than rejected.

    // ---- SYNOPSIS form 1: choom [options] -p pid

    #[test]
    fn query_pid() {
        insta::assert_snapshot!(parse::<Args>(&["choom", "-p", "1234"]));
    }

    // ---- SYNOPSIS form 2: choom [options] -n number -p pid

    #[test]
    fn set_pid_score() {
        insta::assert_snapshot!(parse::<Args>(&[
            "choom", "-n", "500", "-p", "1234",
        ]));
    }

    #[test]
    fn negative_score() {
        insta::assert_snapshot!(parse::<Args>(&[
            "choom", "-n", "-500", "-p", "1234",
        ]));
    }

    // ---- SYNOPSIS form 3: choom [options] -n number [--] command [args...]

    #[test]
    fn run_command_with_score() {
        insta::assert_snapshot!(parse::<Args>(&[
            "choom", "-n", "100", "--", "myprog", "--flag",
        ]));
    }

    #[test]
    fn run_command_no_dash_dash() {
        insta::assert_snapshot!(parse::<Args>(&[
            "choom", "-n", "100", "myprog", "arg1",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_adjust_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "choom", "-n", "abc", "-p", "1234",
        ]));
    }

    #[test]
    fn err_adjust_out_of_range() {
        insta::assert_snapshot!(parse::<Args>(&[
            "choom", "-n", "99999", "-p", "1234",
        ]));
    }
}

mod ctrlaltdel {
    use super::parse;
    use linuxutils_system::ctrlaltdel::Args;

    // ---- SYNOPSIS form: ctrlaltdel [hard|soft]

    #[test]
    fn no_args_query() {
        insta::assert_snapshot!(parse::<Args>(&["ctrlaltdel"]));
    }

    #[test]
    fn hard_mode() {
        insta::assert_snapshot!(parse::<Args>(&["ctrlaltdel", "hard"]));
    }

    #[test]
    fn soft_mode() {
        insta::assert_snapshot!(parse::<Args>(&["ctrlaltdel", "soft"]));
    }

    // Mode is a free-form String, so clap accepts anything here; validation
    // happens at run time.
    #[test]
    fn arbitrary_string_accepted_by_clap() {
        insta::assert_snapshot!(parse::<Args>(&["ctrlaltdel", "bogus"]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["ctrlaltdel", "--bogus"]));
    }
}

mod dmesg {
    use super::parse;
    use linuxutils_system::dmesg::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["dmesg"]));
    }

    #[test]
    fn human_readable() {
        insta::assert_snapshot!(parse::<Args>(&["dmesg", "-H"]));
    }

    #[test]
    fn level_and_facility_filter() {
        insta::assert_snapshot!(parse::<Args>(&[
            "dmesg",
            "-l",
            "err,warn",
            "-f",
            "kern,daemon",
        ]));
    }

    #[test]
    fn kernel_only_with_decode() {
        insta::assert_snapshot!(parse::<Args>(&["dmesg", "-k", "-x"]));
    }

    #[test]
    fn follow_with_color() {
        insta::assert_snapshot!(parse::<Args>(&[
            "dmesg", "-w", "--color", "always",
        ]));
    }

    #[test]
    fn color_no_value() {
        insta::assert_snapshot!(parse::<Args>(&["dmesg", "-L"]));
    }

    #[test]
    fn time_format_iso() {
        insta::assert_snapshot!(parse::<Args>(&[
            "dmesg",
            "--time-format",
            "iso",
        ]));
    }

    #[test]
    fn raw_and_notime() {
        insta::assert_snapshot!(parse::<Args>(&["dmesg", "-r", "-t"]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["dmesg", "--bogus"]));
    }
}

mod eject {
    use super::parse;
    use linuxutils_system::eject::Args;

    // ---- SYNOPSIS form: eject [options] [<device>|<mountpoint>]

    #[test]
    fn no_args_default_device() {
        insta::assert_snapshot!(parse::<Args>(&["eject"]));
    }

    #[test]
    fn explicit_device() {
        insta::assert_snapshot!(parse::<Args>(&["eject", "/dev/sr0"]));
    }

    #[test]
    fn show_default() {
        insta::assert_snapshot!(parse::<Args>(&["eject", "-d"]));
    }

    #[test]
    fn force_cdrom_verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "eject", "-F", "-r", "-v", "/dev/sr0",
        ]));
    }

    #[test]
    fn tray_toggle_close() {
        insta::assert_snapshot!(parse::<Args>(&[
            "eject", "-T", "-t", "/dev/sr0",
        ]));
    }

    #[test]
    fn manualeject_on() {
        insta::assert_snapshot!(parse::<Args>(&[
            "eject", "-i", "on", "/dev/sr0",
        ]));
    }

    #[test]
    fn no_unmount_floppy() {
        insta::assert_snapshot!(parse::<Args>(&[
            "eject", "-m", "-f", "/dev/fd0",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["eject", "--bogus"]));
    }
}

mod fadvise {
    use super::parse;
    use linuxutils_system::fadvise::Args;

    // ---- SYNOPSIS form: fadvise [options] <file>
    //                     fadvise [options] -d <fd>

    #[test]
    fn file_default_advice() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fadvise",
            "/var/log/messages",
        ]));
    }

    #[test]
    fn willneed_with_range() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fadvise", "-a", "willneed", "-o", "0", "-l", "65536", "data.bin",
        ]));
    }

    #[test]
    fn sequential_access() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fadvise",
            "-a",
            "sequential",
            "log.txt",
        ]));
    }

    #[test]
    fn fd_form() {
        insta::assert_snapshot!(parse::<Args>(&["fadvise", "-d", "3"]));
    }

    // ---- Error cases

    #[test]
    fn err_offset_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fadvise", "-o", "abc", "data.bin",
        ]));
    }
}

mod fallocate {
    use super::parse;
    use linuxutils_system::fallocate::Args;

    // ---- SYNOPSIS form: fallocate [options] <filename>

    #[test]
    fn length_and_filename() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fallocate",
            "-l",
            "1M",
            "out.bin",
        ]));
    }

    #[test]
    fn length_with_offset() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fallocate",
            "-l",
            "4K",
            "-o",
            "1024",
            "out.bin",
        ]));
    }

    #[test]
    fn punch_hole() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fallocate",
            "-p",
            "-o",
            "0",
            "-l",
            "4096",
            "data.bin",
        ]));
    }

    #[test]
    fn dig_holes() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fallocate",
            "-d",
            "data.bin",
        ]));
    }

    #[test]
    fn collapse_range_keep_size() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fallocate",
            "-c",
            "-n",
            "-o",
            "0",
            "-l",
            "4096",
            "data.bin",
        ]));
    }

    #[test]
    fn posix_fallocate() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fallocate",
            "-x",
            "-l",
            "1G",
            "out.bin",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_filename() {
        insta::assert_snapshot!(parse::<Args>(&["fallocate", "-l", "1M"]));
    }

    #[test]
    fn err_conflicting_modes() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fallocate",
            "-c",
            "-p",
            "-l",
            "4K",
            "data.bin",
        ]));
    }
}

mod flock {
    use super::parse;
    use linuxutils_system::flock::Args;

    // flock has `args: Vec<String>` with `trailing_var_arg = true,
    // allow_hyphen_values = true`, so unknown flags trailing the file
    // get consumed as positional command args.

    // ---- SYNOPSIS form 1: flock [options] <file> <command> [<arg>...]

    #[test]
    fn file_command() {
        insta::assert_snapshot!(parse::<Args>(&[
            "flock",
            "/var/lock/myapp",
            "mycmd",
        ]));
    }

    #[test]
    fn shared_lock() {
        insta::assert_snapshot!(parse::<Args>(&[
            "flock",
            "-s",
            "/var/lock/myapp",
            "mycmd",
        ]));
    }

    #[test]
    fn nonblock_with_exit_code() {
        insta::assert_snapshot!(parse::<Args>(&[
            "flock",
            "-n",
            "-E",
            "5",
            "/var/lock/myapp",
            "mycmd",
        ]));
    }

    #[test]
    fn timeout_decimal() {
        insta::assert_snapshot!(parse::<Args>(&[
            "flock",
            "-w",
            "1.5",
            "/var/lock/myapp",
            "mycmd",
        ]));
    }

    // ---- SYNOPSIS form 2: flock [options] <file> -c <command>

    #[test]
    fn shell_command_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "flock",
            "/var/lock/myapp",
            "-c",
            "echo hello",
        ]));
    }

    // ---- SYNOPSIS form 3: flock [options] <fd>

    #[test]
    fn fd_form() {
        insta::assert_snapshot!(parse::<Args>(&["flock", "200"]));
    }

    #[test]
    fn close_no_fork() {
        insta::assert_snapshot!(parse::<Args>(&[
            "flock",
            "-o",
            "-F",
            "/var/lock/myapp",
            "mycmd",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_file() {
        insta::assert_snapshot!(parse::<Args>(&["flock"]));
    }

    #[test]
    fn err_timeout_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "flock",
            "-w",
            "abc",
            "/var/lock/myapp",
            "mycmd",
        ]));
    }
}

mod fsfreeze {
    use super::parse;
    use linuxutils_system::fsfreeze::Args;

    // ---- SYNOPSIS form: fsfreeze {-f | -u} <mountpoint>

    #[test]
    fn freeze() {
        insta::assert_snapshot!(parse::<Args>(&["fsfreeze", "-f", "/mnt"]));
    }

    #[test]
    fn unfreeze() {
        insta::assert_snapshot!(parse::<Args>(&["fsfreeze", "-u", "/mnt"]));
    }

    #[test]
    fn freeze_long() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fsfreeze", "--freeze", "/data",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_mountpoint() {
        insta::assert_snapshot!(parse::<Args>(&["fsfreeze", "-f"]));
    }

    #[test]
    fn err_freeze_and_unfreeze_conflict() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fsfreeze", "-f", "-u", "/mnt",
        ]));
    }
}

mod fstrim {
    use super::parse;
    use linuxutils_system::fstrim::Args;

    // ---- SYNOPSIS form: fstrim [options] <mountpoint>

    #[test]
    fn mountpoint_only() {
        insta::assert_snapshot!(parse::<Args>(&["fstrim", "/"]));
    }

    #[test]
    fn all() {
        insta::assert_snapshot!(parse::<Args>(&["fstrim", "-a"]));
    }

    #[test]
    fn fstab() {
        insta::assert_snapshot!(parse::<Args>(&["fstrim", "-A"]));
    }

    #[test]
    fn offset_length_minimum() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fstrim",
            "-o",
            "0",
            "-l",
            "1073741824",
            "-m",
            "4096",
            "/mnt",
        ]));
    }

    #[test]
    fn types_filter_verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "fstrim", "-t", "ext4,xfs", "-v", "-a",
        ]));
    }

    #[test]
    fn dry_run() {
        insta::assert_snapshot!(parse::<Args>(&["fstrim", "-n", "/"]));
    }

    // ---- Error cases

    #[test]
    fn err_minimum_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["fstrim", "-m", "abc", "/",]));
    }
}

mod ipcmk {
    use super::parse;
    use linuxutils_system::ipcmk::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["ipcmk"]));
    }

    #[test]
    fn shared_memory() {
        insta::assert_snapshot!(parse::<Args>(&["ipcmk", "-M", "65536",]));
    }

    #[test]
    fn message_queue() {
        insta::assert_snapshot!(parse::<Args>(&["ipcmk", "-Q"]));
    }

    #[test]
    fn semaphore() {
        insta::assert_snapshot!(parse::<Args>(&["ipcmk", "-S", "16"]));
    }

    #[test]
    fn shared_memory_with_mode() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ipcmk", "-M", "1024", "-p", "0600",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_shmem_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["ipcmk", "-M", "abc"]));
    }

    #[test]
    fn err_semaphore_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["ipcmk", "-S", "abc"]));
    }
}

mod ipcrm {
    use super::parse;
    use linuxutils_system::ipcrm::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["ipcrm"]));
    }

    #[test]
    fn shmem_id() {
        insta::assert_snapshot!(parse::<Args>(&["ipcrm", "-m", "12345"]));
    }

    #[test]
    fn shmem_key_hex() {
        insta::assert_snapshot!(parse::<Args>(&["ipcrm", "-M", "0x1234abcd",]));
    }

    #[test]
    fn multiple_resources() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ipcrm", "-q", "100", "-q", "200", "-s", "300",
        ]));
    }

    #[test]
    fn all_default() {
        insta::assert_snapshot!(parse::<Args>(&["ipcrm", "-a"]));
    }

    #[test]
    fn all_specific_kind() {
        insta::assert_snapshot!(parse::<Args>(&["ipcrm", "-a", "shm"]));
    }

    #[test]
    fn verbose_with_targets() {
        insta::assert_snapshot!(parse::<Args>(&[
            "ipcrm",
            "-v",
            "-Q",
            "0xdeadbeef",
        ]));
    }
}

mod ipcs {
    use super::parse;
    use linuxutils_system::ipcs::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs"]));
    }

    #[test]
    fn shared_memory() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs", "-m"]));
    }

    #[test]
    fn message_queues() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs", "-q"]));
    }

    #[test]
    fn semaphores() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs", "-s"]));
    }

    #[test]
    fn all() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs", "-a"]));
    }

    #[test]
    fn limits() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs", "-l"]));
    }

    #[test]
    fn summary_bytes() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs", "-u", "-b"]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["ipcs", "--bogus"]));
    }
}

mod losetup {
    use super::parse;
    use linuxutils_system::losetup::Args;

    #[test]
    fn list_all() {
        insta::assert_snapshot!(parse::<Args>(&["losetup", "-a"]));
    }

    #[test]
    fn find_free() {
        insta::assert_snapshot!(parse::<Args>(&["losetup", "-f"]));
    }

    #[test]
    fn find_and_show_with_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "losetup", "-f", "--show", "disk.img",
        ]));
    }

    #[test]
    fn detach() {
        insta::assert_snapshot!(parse::<Args>(&[
            "losetup",
            "-d",
            "/dev/loop0",
        ]));
    }

    #[test]
    fn detach_all() {
        insta::assert_snapshot!(parse::<Args>(&["losetup", "-D"]));
    }

    #[test]
    fn associate_with_offset() {
        insta::assert_snapshot!(parse::<Args>(&[
            "losetup",
            "-o",
            "1048576",
            "--sizelimit",
            "10485760",
            "/dev/loop0",
            "disk.img",
        ]));
    }

    #[test]
    fn associated_filter() {
        insta::assert_snapshot!(parse::<Args>(&[
            "losetup",
            "-j",
            "/path/to/file.img",
        ]));
    }

    #[test]
    fn list_table_format() {
        insta::assert_snapshot!(parse::<Args>(&["losetup", "-l", "-n",]));
    }

    #[test]
    fn read_only_partscan() {
        insta::assert_snapshot!(parse::<Args>(&[
            "losetup",
            "-r",
            "-P",
            "/dev/loop0",
            "disk.img",
        ]));
    }
}

mod lscpu {
    use super::parse;
    use linuxutils_system::lscpu::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["lscpu"]));
    }

    #[test]
    fn bytes() {
        insta::assert_snapshot!(parse::<Args>(&["lscpu", "-B"]));
    }

    #[test]
    fn bytes_long() {
        insta::assert_snapshot!(parse::<Args>(&["lscpu", "--bytes"]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["lscpu", "--bogus"]));
    }
}

mod lsipc {
    use super::parse;
    use linuxutils_system::lsipc::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["lsipc"]));
    }

    #[test]
    fn shared_memory() {
        insta::assert_snapshot!(parse::<Args>(&["lsipc", "-m"]));
    }

    #[test]
    fn message_queues() {
        insta::assert_snapshot!(parse::<Args>(&["lsipc", "-q"]));
    }

    #[test]
    fn semaphores() {
        insta::assert_snapshot!(parse::<Args>(&["lsipc", "-s"]));
    }

    #[test]
    fn global_summary() {
        insta::assert_snapshot!(parse::<Args>(&["lsipc", "-g"]));
    }

    #[test]
    fn noheadings_bytes() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lsipc",
            "-m",
            "--noheadings",
            "-b",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["lsipc", "--bogus"]));
    }
}

mod lsirq {
    use super::parse;
    use linuxutils_system::lsirq::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["lsirq"]));
    }

    #[test]
    fn json_output() {
        insta::assert_snapshot!(parse::<Args>(&["lsirq", "-J"]));
    }

    #[test]
    fn pairs_noheadings() {
        insta::assert_snapshot!(parse::<Args>(&["lsirq", "-P", "-n"]));
    }

    #[test]
    fn output_columns_sort() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lsirq",
            "-o",
            "IRQ,TOTAL,NAME",
            "-s",
            "TOTAL",
        ]));
    }

    #[test]
    fn softirq_threshold() {
        insta::assert_snapshot!(parse::<Args>(&["lsirq", "-S", "-t", "100",]));
    }

    #[test]
    fn cpu_list() {
        insta::assert_snapshot!(parse::<Args>(&["lsirq", "-C", "0,1,2,3",]));
    }
}

mod lsmem {
    use super::parse;
    use linuxutils_system::lsmem::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["lsmem"]));
    }

    #[test]
    fn all_blocks() {
        insta::assert_snapshot!(parse::<Args>(&["lsmem", "-a"]));
    }

    #[test]
    fn json_bytes() {
        insta::assert_snapshot!(parse::<Args>(&["lsmem", "-J", "-b"]));
    }

    #[test]
    fn output_columns() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lsmem",
            "-o",
            "RANGE,SIZE,STATE",
        ]));
    }

    #[test]
    fn raw_output_all() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lsmem",
            "-r",
            "--output-all",
        ]));
    }

    #[test]
    fn split_columns() {
        insta::assert_snapshot!(parse::<Args>(&["lsmem", "-S", "STATE,NODE",]));
    }

    #[test]
    fn sysroot_summary() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lsmem",
            "-s",
            "/sysroot",
            "--summary",
            "only",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_invalid_summary() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lsmem",
            "--summary",
            "bogus",
        ]));
    }
}

mod lsns {
    use super::parse;
    use linuxutils_system::lsns::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["lsns"]));
    }

    #[test]
    fn type_filter() {
        insta::assert_snapshot!(parse::<Args>(&["lsns", "-t", "net"]));
    }

    #[test]
    fn task_pid() {
        insta::assert_snapshot!(parse::<Args>(&["lsns", "-p", "1234"]));
    }

    #[test]
    fn json_output_all() {
        insta::assert_snapshot!(parse::<Args>(
            &["lsns", "-J", "--output-all",]
        ));
    }

    #[test]
    fn output_columns_raw() {
        insta::assert_snapshot!(parse::<Args>(&[
            "lsns",
            "-o",
            "NS,TYPE,PID,COMMAND",
            "-r",
        ]));
    }

    #[test]
    fn list_noheadings() {
        insta::assert_snapshot!(parse::<Args>(&["lsns", "-l", "-n"]));
    }

    #[test]
    fn namespace_inode() {
        insta::assert_snapshot!(parse::<Args>(&["lsns", "4026531840",]));
    }

    // ---- Error cases

    #[test]
    fn err_task_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["lsns", "-p", "abc"]));
    }
}

mod mountpoint {
    use super::parse;
    use linuxutils_system::mountpoint::Args;

    // ---- SYNOPSIS form: mountpoint [options] <directory>|<file>

    #[test]
    fn directory_only() {
        insta::assert_snapshot!(parse::<Args>(&["mountpoint", "/mnt"]));
    }

    #[test]
    fn quiet() {
        insta::assert_snapshot!(parse::<Args>(&["mountpoint", "-q", "/mnt",]));
    }

    #[test]
    fn fs_devno() {
        insta::assert_snapshot!(parse::<Args>(&["mountpoint", "-d", "/mnt",]));
    }

    #[test]
    fn devno_block_device() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mountpoint",
            "-x",
            "/dev/sda1",
        ]));
    }

    #[test]
    fn nofollow() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mountpoint",
            "--nofollow",
            "/mnt/link",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_path() {
        insta::assert_snapshot!(parse::<Args>(&["mountpoint"]));
    }

    #[test]
    fn err_devno_conflicts_with_quiet() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mountpoint",
            "-x",
            "-q",
            "/dev/sda1",
        ]));
    }

    #[test]
    fn err_devno_conflicts_with_fs_devno() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mountpoint",
            "-d",
            "-x",
            "/dev/sda1",
        ]));
    }
}

mod nsenter {
    use super::parse;
    use linuxutils_system::nsenter::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["nsenter"]));
    }

    #[test]
    fn target_pid_all() {
        insta::assert_snapshot!(parse::<Args>(&[
            "nsenter", "-t", "1234", "-a",
        ]));
    }

    #[test]
    fn target_with_command() {
        insta::assert_snapshot!(parse::<Args>(&[
            "nsenter", "-t", "1234", "-m", "-n", "-u", "ls",
        ]));
    }

    #[test]
    fn explicit_ns_paths() {
        insta::assert_snapshot!(parse::<Args>(&[
            "nsenter",
            "--mount=/proc/1234/ns/mnt",
            "--net=/proc/1234/ns/net",
            "ls",
        ]));
    }

    #[test]
    fn no_fork_preserve_credentials() {
        insta::assert_snapshot!(parse::<Args>(&[
            "nsenter",
            "-t",
            "1234",
            "-p",
            "-F",
            "--preserve-credentials",
            "ls",
        ]));
    }

    #[test]
    fn root_workdir_setuid_setgid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "nsenter", "-t", "1234", "-r", "/", "-w", "/tmp", "-S", "1000",
            "-G", "1000", "ls",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_target_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["nsenter", "-t", "abc",]));
    }
}

mod pipesz {
    use super::parse;
    use linuxutils_system::pipesz::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["pipesz"]));
    }

    #[test]
    fn get_stdin() {
        insta::assert_snapshot!(parse::<Args>(&["pipesz", "-g", "-i"]));
    }

    #[test]
    fn set_size_with_fd() {
        insta::assert_snapshot!(parse::<Args>(&[
            "pipesz", "-s", "1048576", "-n", "3",
        ]));
    }

    #[test]
    fn set_with_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "pipesz",
            "-s",
            "65536",
            "-f",
            "/tmp/myfifo",
        ]));
    }

    #[test]
    fn stdout_stderr() {
        insta::assert_snapshot!(parse::<Args>(&["pipesz", "-g", "-o", "-e",]));
    }

    #[test]
    fn set_with_command() {
        insta::assert_snapshot!(parse::<Args>(&[
            "pipesz", "-s", "65536", "myprog", "arg1",
        ]));
    }

    #[test]
    fn check_quiet_verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "pipesz", "-c", "-q", "-v", "-g",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_set_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["pipesz", "-s", "abc"]));
    }
}

mod pivot_root {
    use super::parse;
    use linuxutils_system::pivot_root::Args;

    // ---- SYNOPSIS form: pivot_root <new_root> <put_old>

    #[test]
    fn basic() {
        insta::assert_snapshot!(parse::<Args>(&[
            "pivot_root",
            "/mnt/newroot",
            "/mnt/newroot/oldroot",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_args() {
        insta::assert_snapshot!(parse::<Args>(&["pivot_root"]));
    }

    #[test]
    fn err_missing_put_old() {
        insta::assert_snapshot!(parse::<Args>(
            &["pivot_root", "/mnt/newroot",]
        ));
    }

    #[test]
    fn err_too_many_args() {
        insta::assert_snapshot!(parse::<Args>(&[
            "pivot_root",
            "/a",
            "/b",
            "/c",
        ]));
    }
}

mod prlimit {
    use super::parse;
    use linuxutils_system::prlimit::Args;

    // prlimit has `command: Vec<String>` with `trailing_var_arg = true,
    // allow_hyphen_values = true`, so unknown flags after a command get
    // consumed as positional args rather than rejected.

    // ---- SYNOPSIS form 1: prlimit [options] [--<resource>=<limit>] [-p PID]

    #[test]
    fn show_all_for_pid() {
        insta::assert_snapshot!(parse::<Args>(&["prlimit", "-p", "1234"]));
    }

    #[test]
    fn show_single_resource_for_pid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "prlimit", "-p", "1234", "--nofile",
        ]));
    }

    #[test]
    fn set_resource_for_pid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "prlimit",
            "-p",
            "1234",
            "--nofile=4096",
        ]));
    }

    #[test]
    fn set_soft_hard_pair() {
        insta::assert_snapshot!(parse::<Args>(&[
            "prlimit",
            "-p",
            "1234",
            "--stack=8388608:16777216",
        ]));
    }

    // ---- SYNOPSIS form 2: prlimit [options] [--<resource>=<limit>] COMMAND [args...]

    #[test]
    fn run_command_with_limits() {
        insta::assert_snapshot!(parse::<Args>(&[
            "prlimit",
            "--nofile=2048",
            "--cpu=60",
            "myprog",
            "arg1",
        ]));
    }

    #[test]
    fn output_columns_raw() {
        insta::assert_snapshot!(parse::<Args>(&[
            "prlimit",
            "-p",
            "1234",
            "-o",
            "RESOURCE,SOFT,HARD",
            "--raw",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_pid_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["prlimit", "-p", "abc",]));
    }

    #[test]
    fn unknown_flag_consumed_as_positional() {
        insta::assert_snapshot!(parse::<Args>(&[
            "prlimit", "myprog", "--bogus",
        ]));
    }
}

mod renice {
    use super::parse;
    use linuxutils_system::renice::Args;

    // renice has `args: Vec<String>` with `allow_hyphen_values = true`,
    // so unknown flags end up consumed as positional arguments.

    // ---- SYNOPSIS form 1: renice [-n priority] [-p|--pid] pid...

    #[test]
    fn priority_pid_implicit() {
        insta::assert_snapshot!(parse::<Args>(&["renice", "10", "1234"]));
    }

    #[test]
    fn priority_negative_pid() {
        insta::assert_snapshot!(parse::<Args>(&["renice", "-5", "1234",]));
    }

    #[test]
    fn priority_flag_pid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "renice", "-n", "10", "-p", "1234",
        ]));
    }

    #[test]
    fn relative_priority() {
        insta::assert_snapshot!(parse::<Args>(&[
            "renice",
            "-n",
            "5",
            "--relative",
            "1234",
        ]));
    }

    // ---- SYNOPSIS form 2: renice [-n priority] -g|--pgrp pgid...

    #[test]
    fn pgrp_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "renice", "-n", "5", "-g", "100", "200",
        ]));
    }

    // ---- SYNOPSIS form 3: renice [-n priority] -u|--user user...

    #[test]
    fn user_by_name() {
        insta::assert_snapshot!(parse::<Args>(&[
            "renice", "-n", "5", "-u", "alice", "bob",
        ]));
    }

    #[test]
    fn user_by_uid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "renice", "-n", "5", "-u", "1000",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_args() {
        insta::assert_snapshot!(parse::<Args>(&["renice"]));
    }
}

mod rfkill {
    use super::parse;
    use linuxutils_system::rfkill::Args;

    // rfkill is a Subcommand-based parser. Without a subcommand, the
    // default is "list as table" using the global options.

    // ---- Default table form

    #[test]
    fn no_subcommand() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill"]));
    }

    #[test]
    fn json_output() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "-J"]));
    }

    #[test]
    fn output_columns_raw() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rfkill",
            "-o",
            "ID,TYPE,DEVICE,SOFT,HARD",
            "-r",
        ]));
    }

    // ---- Subcommand: event

    #[test]
    fn event_subcommand() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "event"]));
    }

    // ---- Subcommand: list

    #[test]
    fn list_no_filter() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "list"]));
    }

    #[test]
    fn list_with_type() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "list", "wlan",]));
    }

    // ---- Subcommand: block

    #[test]
    fn block_by_id() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "block", "0"]));
    }

    #[test]
    fn block_by_type() {
        insta::assert_snapshot!(parse::<Args>(&[
            "rfkill",
            "block",
            "bluetooth",
        ]));
    }

    // ---- Subcommand: unblock

    #[test]
    fn unblock_all() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "unblock", "all",]));
    }

    // ---- Subcommand: toggle

    #[test]
    fn toggle_wifi() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "toggle", "wifi",]));
    }

    // ---- Error cases

    #[test]
    fn err_block_missing_identifier() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "block"]));
    }

    #[test]
    fn err_unknown_subcommand() {
        insta::assert_snapshot!(parse::<Args>(&["rfkill", "bogus"]));
    }
}

mod setarch {
    use super::parse;
    use linuxutils_system::setarch::Args;

    // ---- SYNOPSIS forms:
    //   setarch [arch] [opts] [program [args]]
    //   setarch --list
    //   setarch --show[=<arch>]

    #[test]
    fn list_only() {
        insta::assert_snapshot!(parse::<Args>(&["setarch", "--list"]));
    }

    #[test]
    fn show_no_value() {
        insta::assert_snapshot!(parse::<Args>(&["setarch", "--show"]));
    }

    #[test]
    fn show_with_value() {
        insta::assert_snapshot!(parse::<Args>(&["setarch", "--show=linux32",]));
    }

    #[test]
    fn arch_and_command() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setarch", "linux32", "uname", "-m",
        ]));
    }

    #[test]
    fn personality_flags() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setarch",
            "--addr-no-randomize",
            "--uname-2.6",
            "linux64",
            "/bin/echo",
            "hi",
        ]));
    }

    #[test]
    fn flag_aliases() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setarch", "-R", "-S", "-3", "linux64", "prog",
        ]));
    }

    #[test]
    fn verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setarch", "-v", "linux64", "prog",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["setarch", "--bogus"]));
    }
}

mod setpgid {
    use super::parse;
    use linuxutils_system::setpgid::Args;

    // setpgid has `command: Vec<String>` with `trailing_var_arg = true,
    // allow_hyphen_values = true, required = true`, so unknown flags
    // before a command get consumed as positionals.

    #[test]
    fn command_only() {
        insta::assert_snapshot!(parse::<Args>(&["setpgid", "myprog"]));
    }

    #[test]
    fn command_with_args() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setpgid", "myprog", "--flag", "arg",
        ]));
    }

    #[test]
    fn foreground() {
        insta::assert_snapshot!(parse::<Args>(&["setpgid", "-f", "myprog",]));
    }

    #[test]
    fn foreground_long() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setpgid",
            "--foreground",
            "myprog",
            "arg",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_command() {
        insta::assert_snapshot!(parse::<Args>(&["setpgid"]));
    }

    #[test]
    fn unknown_flag_consumed_as_positional() {
        insta::assert_snapshot!(parse::<Args>(&["setpgid", "--bogus"]));
    }
}

mod setsid {
    use super::parse;
    use linuxutils_system::setsid::Args;

    // setsid has `command: Vec<String>` with `trailing_var_arg = true,
    // allow_hyphen_values = true, required = true`.

    #[test]
    fn command_only() {
        insta::assert_snapshot!(parse::<Args>(&["setsid", "myprog"]));
    }

    #[test]
    fn ctty() {
        insta::assert_snapshot!(parse::<Args>(&["setsid", "-c", "myprog",]));
    }

    #[test]
    fn fork() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setsid", "-f", "myprog", "arg1",
        ]));
    }

    #[test]
    fn wait() {
        insta::assert_snapshot!(parse::<Args>(&["setsid", "-w", "myprog",]));
    }

    #[test]
    fn all_flags() {
        insta::assert_snapshot!(parse::<Args>(&[
            "setsid", "-c", "-f", "-w", "myprog",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_missing_command() {
        insta::assert_snapshot!(parse::<Args>(&["setsid"]));
    }

    #[test]
    fn unknown_flag_consumed_as_positional() {
        insta::assert_snapshot!(parse::<Args>(&["setsid", "--bogus"]));
    }
}

mod swapoff {
    use super::parse;
    use linuxutils_system::swapoff::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["swapoff"]));
    }

    #[test]
    fn all() {
        insta::assert_snapshot!(parse::<Args>(&["swapoff", "-a"]));
    }

    #[test]
    fn single_device() {
        insta::assert_snapshot!(parse::<Args>(&["swapoff", "/dev/sda2",]));
    }

    #[test]
    fn multiple_devices() {
        insta::assert_snapshot!(parse::<Args>(&[
            "swapoff",
            "/dev/sda2",
            "/swapfile",
        ]));
    }

    #[test]
    fn verbose_all() {
        insta::assert_snapshot!(parse::<Args>(&["swapoff", "-v", "-a"]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["swapoff", "--bogus"]));
    }
}

mod swapon {
    use super::parse;
    use linuxutils_system::swapon::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["swapon"]));
    }

    #[test]
    fn all() {
        insta::assert_snapshot!(parse::<Args>(&["swapon", "-a"]));
    }

    #[test]
    fn single_device() {
        insta::assert_snapshot!(parse::<Args>(&["swapon", "/dev/sda2",]));
    }

    #[test]
    fn priority_and_discard() {
        insta::assert_snapshot!(parse::<Args>(&[
            "swapon",
            "-p",
            "10",
            "-d",
            "pages",
            "/swapfile",
        ]));
    }

    #[test]
    fn discard_no_value() {
        insta::assert_snapshot!(parse::<Args>(&["swapon", "-d", "/swapfile",]));
    }

    #[test]
    fn summary() {
        insta::assert_snapshot!(parse::<Args>(&["swapon", "-s"]));
    }

    #[test]
    fn show_with_columns() {
        insta::assert_snapshot!(parse::<Args>(&[
            "swapon",
            "--show",
            "NAME,TYPE,SIZE",
            "--noheadings",
            "--bytes",
        ]));
    }

    #[test]
    fn ifexists_verbose() {
        insta::assert_snapshot!(parse::<Args>(&["swapon", "-e", "-v", "-a",]));
    }

    // ---- Error cases

    #[test]
    fn err_priority_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "swapon",
            "-p",
            "abc",
            "/swapfile",
        ]));
    }
}

mod wdctl {
    use super::parse;
    use linuxutils_system::wdctl::Args;

    // ---- SYNOPSIS form: wdctl [options] [device...]

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["wdctl"]));
    }

    #[test]
    fn explicit_device_via_d() {
        insta::assert_snapshot!(parse::<Args>(&[
            "wdctl",
            "-d",
            "/dev/watchdog1",
        ]));
    }

    #[test]
    fn positional_devices() {
        insta::assert_snapshot!(parse::<Args>(&[
            "wdctl",
            "/dev/watchdog0",
            "/dev/watchdog1",
        ]));
    }

    #[test]
    fn flags_only() {
        insta::assert_snapshot!(parse::<Args>(&["wdctl", "-x"]));
    }

    #[test]
    fn no_subsections() {
        insta::assert_snapshot!(parse::<Args>(&[
            "wdctl", "-I", "-T", "-F", "-n",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["wdctl", "--bogus"]));
    }
}

mod unshare {
    use super::parse;
    use linuxutils_system::unshare::Args;

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["unshare"]));
    }

    #[test]
    fn mount_uts_ipc() {
        insta::assert_snapshot!(parse::<Args>(&["unshare", "-m", "-u", "-i",]));
    }

    #[test]
    fn user_map_root_with_command() {
        insta::assert_snapshot!(parse::<Args>(&[
            "unshare", "-U", "-r", "-f", "bash",
        ]));
    }

    #[test]
    fn map_current_user() {
        insta::assert_snapshot!(parse::<Args>(&["unshare", "-c", "bash",]));
    }

    #[test]
    fn pid_mount_proc_default() {
        insta::assert_snapshot!(parse::<Args>(&[
            "unshare",
            "-p",
            "-f",
            "--mount-proc",
            "bash",
        ]));
    }

    #[test]
    fn pid_mount_proc_explicit() {
        insta::assert_snapshot!(parse::<Args>(&[
            "unshare",
            "-p",
            "-f",
            "--mount-proc=/myproc",
            "bash",
        ]));
    }

    #[test]
    fn propagation_shared() {
        insta::assert_snapshot!(parse::<Args>(&[
            "unshare",
            "-m",
            "--propagation",
            "shared",
            "bash",
        ]));
    }

    #[test]
    fn root_workdir_setuid() {
        insta::assert_snapshot!(parse::<Args>(&[
            "unshare", "-m", "-R", "/newroot", "-w", "/tmp", "-S", "1000",
            "-G", "1000", "bash",
        ]));
    }

    #[test]
    fn setgroups_deny() {
        insta::assert_snapshot!(parse::<Args>(&[
            "unshare",
            "-U",
            "--setgroups",
            "deny",
            "bash",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_setuid_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&["unshare", "-S", "abc",]));
    }
}
