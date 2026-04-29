use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::{
    process::Pid,
    thread::{CpuSet, sched_getaffinity, sched_setaffinity},
};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "taskset",
    about = "Set or retrieve a process's CPU affinity",
    override_usage = "taskset [options] [mask] command [arg...]\n       \
                      taskset [options] -p [mask] pid"
)]
pub struct Args {
    /// Operate on an existing PID
    #[arg(short = 'p', long = "pid")]
    pid: bool,

    /// Interpret mask as CPU list instead of bitmask
    #[arg(short = 'c', long = "cpu-list")]
    cpu_list: bool,

    /// Operate on all tasks (threads) for a given PID
    #[arg(short = 'a', long = "all-tasks")]
    all_tasks: bool,

    /// Positional arguments: mask and/or PID/command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn parse_hex_mask(s: &str) -> Result<CpuSet, String> {
    let hex = s.strip_prefix("0x").or(s.strip_prefix("0X")).unwrap_or(s);
    if hex.is_empty() {
        return Err("empty mask".to_string());
    }
    let mut cpuset = CpuSet::new();
    let chars: Vec<char> = hex.chars().collect();
    let num_nibbles = chars.len();
    for (i, ch) in chars.iter().enumerate() {
        let nibble = ch
            .to_digit(16)
            .ok_or_else(|| format!("invalid hex character: '{ch}'"))?;
        let bit_offset = (num_nibbles - 1 - i) * 4;
        for bit in 0..4 {
            if nibble & (1 << bit) != 0 {
                let cpu = bit_offset + bit;
                if cpu >= CpuSet::MAX_CPU {
                    return Err(format!(
                        "CPU {cpu} exceeds maximum {}",
                        CpuSet::MAX_CPU - 1
                    ));
                }
                cpuset.set(cpu);
            }
        }
    }
    Ok(cpuset)
}

fn parse_cpu_list(s: &str) -> Result<CpuSet, String> {
    let mut cpuset = CpuSet::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((range, stride_str)) = part.split_once(':') {
            let stride: usize = stride_str
                .parse()
                .map_err(|_| format!("invalid stride: '{stride_str}'"))?;
            if stride == 0 {
                return Err("stride cannot be zero".to_string());
            }
            let (start, end) = parse_range(range)?;
            let mut cpu = start;
            while cpu <= end {
                set_cpu(&mut cpuset, cpu)?;
                cpu += stride;
            }
        } else if let Some((start_s, end_s)) = part.split_once('-') {
            let start: usize = start_s
                .trim()
                .parse()
                .map_err(|_| format!("invalid CPU number: '{start_s}'"))?;
            let end: usize = end_s
                .trim()
                .parse()
                .map_err(|_| format!("invalid CPU number: '{end_s}'"))?;
            if start > end {
                return Err(format!("invalid range: {start}-{end}"));
            }
            for cpu in start..=end {
                set_cpu(&mut cpuset, cpu)?;
            }
        } else {
            let cpu: usize = part
                .parse()
                .map_err(|_| format!("invalid CPU number: '{part}'"))?;
            set_cpu(&mut cpuset, cpu)?;
        }
    }
    Ok(cpuset)
}

fn parse_range(s: &str) -> Result<(usize, usize), String> {
    let (start_s, end_s) = s
        .split_once('-')
        .ok_or_else(|| format!("invalid range: '{s}'"))?;
    let start: usize = start_s
        .trim()
        .parse()
        .map_err(|_| format!("invalid CPU number: '{start_s}'"))?;
    let end: usize = end_s
        .trim()
        .parse()
        .map_err(|_| format!("invalid CPU number: '{end_s}'"))?;
    if start > end {
        return Err(format!("invalid range: {start}-{end}"));
    }
    Ok((start, end))
}

fn set_cpu(cpuset: &mut CpuSet, cpu: usize) -> Result<(), String> {
    if cpu >= CpuSet::MAX_CPU {
        return Err(format!(
            "CPU {cpu} exceeds maximum {}",
            CpuSet::MAX_CPU - 1
        ));
    }
    cpuset.set(cpu);
    Ok(())
}

fn cpuset_is_empty(cpuset: &CpuSet) -> bool {
    cpuset.count() == 0
}

fn format_hex_mask(cpuset: &CpuSet) -> String {
    let mut max_cpu = 0;
    for i in 0..CpuSet::MAX_CPU {
        if cpuset.is_set(i) {
            max_cpu = i;
        }
    }
    let num_nibbles = max_cpu / 4 + 1;
    let mut result = String::new();
    for i in (0..num_nibbles).rev() {
        let mut nibble = 0u8;
        for bit in 0..4 {
            let cpu = i * 4 + bit;
            if cpu < CpuSet::MAX_CPU && cpuset.is_set(cpu) {
                nibble |= 1 << bit;
            }
        }
        result.push(char::from_digit(nibble as u32, 16).unwrap());
    }
    if result.is_empty() {
        "0".to_string()
    } else {
        result
    }
}

fn format_cpu_list(cpuset: &CpuSet) -> String {
    let mut cpus: Vec<usize> = Vec::new();
    for i in 0..CpuSet::MAX_CPU {
        if cpuset.is_set(i) {
            cpus.push(i);
        }
    }
    if cpus.is_empty() {
        return String::new();
    }
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < cpus.len() {
        let start = cpus[i];
        let mut end = start;
        while i + 1 < cpus.len() && cpus[i + 1] == end + 1 {
            end = cpus[i + 1];
            i += 1;
        }
        if start == end {
            parts.push(format!("{start}"));
        } else {
            parts.push(format!("{start}-{end}"));
        }
        i += 1;
    }
    parts.join(",")
}

fn parse_mask(s: &str, cpu_list: bool) -> Result<CpuSet, String> {
    if cpu_list {
        parse_cpu_list(s)
    } else {
        parse_hex_mask(s)
    }
}

fn affinity_label(cpu_list: bool) -> &'static str {
    if cpu_list {
        "affinity list"
    } else {
        "affinity mask"
    }
}

fn format_affinity(cpuset: &CpuSet, cpu_list: bool) -> String {
    if cpu_list {
        format_cpu_list(cpuset)
    } else {
        format_hex_mask(cpuset)
    }
}

fn get_task_tids(pid: i32) -> Result<Vec<i32>, std::io::Error> {
    let task_dir = format!("/proc/{pid}/task");
    let mut tids = Vec::new();
    for entry in std::fs::read_dir(&task_dir)? {
        let entry = entry?;
        if let Ok(tid) = entry.file_name().to_string_lossy().parse::<i32>() {
            tids.push(tid);
        }
    }
    Ok(tids)
}

fn do_get_affinity(pid_val: i32, cpu_list: bool) -> Result<(), String> {
    let pid = Pid::from_raw(pid_val);
    let cpuset = sched_getaffinity(pid)
        .map_err(|e| format!("failed to get pid {pid_val}'s affinity: {e}"))?;
    let label = affinity_label(cpu_list);
    let formatted = format_affinity(&cpuset, cpu_list);
    println!("pid {pid_val}'s current {label}: {formatted}");
    Ok(())
}

fn do_set_affinity(
    pid_val: i32,
    new_cpuset: &CpuSet,
    cpu_list: bool,
) -> Result<(), String> {
    let pid = Pid::from_raw(pid_val);
    let label = affinity_label(cpu_list);

    let old_cpuset = sched_getaffinity(pid)
        .map_err(|e| format!("failed to get pid {pid_val}'s affinity: {e}"))?;
    let old_formatted = format_affinity(&old_cpuset, cpu_list);

    sched_setaffinity(pid, new_cpuset)
        .map_err(|e| format!("failed to set pid {pid_val}'s affinity: {e}"))?;

    let cur_cpuset = sched_getaffinity(pid)
        .map_err(|e| format!("failed to get pid {pid_val}'s affinity: {e}"))?;
    let new_formatted = format_affinity(&cur_cpuset, cpu_list);

    println!("pid {pid_val}'s current {label}: {old_formatted}");
    println!("pid {pid_val}'s new {label}: {new_formatted}");
    Ok(())
}

pub fn run(args: Args) -> ExitCode {
    if args.pid {
        return run_pid_mode(&args);
    }
    run_command_mode(&args)
}

fn run_pid_mode(args: &Args) -> ExitCode {
    let positional = &args.args;

    if positional.is_empty() {
        eprintln!("taskset: missing PID argument");
        return ExitCode::FAILURE;
    }

    if positional.len() == 1 {
        let pid: i32 = match positional[0].parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("taskset: invalid PID: '{}'", positional[0]);
                return ExitCode::FAILURE;
            }
        };
        if args.all_tasks {
            match get_task_tids(pid) {
                Ok(tids) => {
                    for tid in tids {
                        match do_get_affinity(tid, args.cpu_list) {
                            Ok(()) => {}
                            Err(e) if e.contains("No such process") => continue,
                            Err(e) => {
                                eprintln!("taskset: {e}");
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "taskset: failed to read tasks for pid {pid}: {e}"
                    );
                    return ExitCode::FAILURE;
                }
            }
        } else if let Err(e) = do_get_affinity(pid, args.cpu_list) {
            eprintln!("taskset: {e}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let mask_str = &positional[0];
    let pid: i32 = match positional[1].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("taskset: invalid PID: '{}'", positional[1]);
            return ExitCode::FAILURE;
        }
    };

    let cpuset = match parse_mask(mask_str, args.cpu_list) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("taskset: failed to parse mask: {e}");
            return ExitCode::FAILURE;
        }
    };

    if cpuset_is_empty(&cpuset) {
        eprintln!("taskset: empty CPU mask is not allowed");
        return ExitCode::FAILURE;
    }

    let tids = if args.all_tasks {
        match get_task_tids(pid) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("taskset: failed to read tasks for pid {pid}: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        vec![pid]
    };

    for tid in tids {
        match do_set_affinity(tid, &cpuset, args.cpu_list) {
            Ok(()) => {}
            Err(e) if args.all_tasks && e.contains("No such process") => {
                continue;
            }
            Err(e) => {
                eprintln!("taskset: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_command_mode(args: &Args) -> ExitCode {
    let positional = &args.args;

    if positional.is_empty() {
        eprintln!("taskset: missing mask argument");
        return ExitCode::FAILURE;
    }

    if positional.len() < 2 {
        eprintln!("taskset: missing command argument");
        return ExitCode::FAILURE;
    }

    let mask_str = &positional[0];
    let cpuset = match parse_mask(mask_str, args.cpu_list) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("taskset: failed to parse mask: {e}");
            return ExitCode::FAILURE;
        }
    };

    if cpuset_is_empty(&cpuset) {
        eprintln!("taskset: empty CPU mask is not allowed");
        return ExitCode::FAILURE;
    }

    if let Err(e) = sched_setaffinity(None, &cpuset) {
        eprintln!("taskset: failed to set affinity: {e}");
        return ExitCode::FAILURE;
    }

    let command = &positional[1..];
    let err = exec_command(command);
    eprintln!("taskset: failed to execute {}: {err}", command[0]);
    ExitCode::FAILURE
}

fn exec_command(command: &[String]) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    std::process::Command::new(&command[0])
        .args(&command[1..])
        .exec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_mask_simple() {
        let cpuset = parse_hex_mask("f").unwrap();
        for i in 0..4 {
            assert!(cpuset.is_set(i));
        }
        assert!(!cpuset.is_set(4));
    }

    #[test]
    fn test_parse_hex_mask_with_prefix() {
        let cpuset = parse_hex_mask("0x3").unwrap();
        assert!(cpuset.is_set(0));
        assert!(cpuset.is_set(1));
        assert!(!cpuset.is_set(2));
    }

    #[test]
    fn test_parse_hex_mask_ff() {
        let cpuset = parse_hex_mask("0xFF").unwrap();
        for i in 0..8 {
            assert!(cpuset.is_set(i));
        }
        assert!(!cpuset.is_set(8));
    }

    #[test]
    fn test_parse_cpu_list_single() {
        let cpuset = parse_cpu_list("0").unwrap();
        assert!(cpuset.is_set(0));
        assert!(!cpuset.is_set(1));
    }

    #[test]
    fn test_parse_cpu_list_range() {
        let cpuset = parse_cpu_list("0-3").unwrap();
        for i in 0..4 {
            assert!(cpuset.is_set(i));
        }
        assert!(!cpuset.is_set(4));
    }

    #[test]
    fn test_parse_cpu_list_comma() {
        let cpuset = parse_cpu_list("0,2,4").unwrap();
        assert!(cpuset.is_set(0));
        assert!(!cpuset.is_set(1));
        assert!(cpuset.is_set(2));
        assert!(!cpuset.is_set(3));
        assert!(cpuset.is_set(4));
    }

    #[test]
    fn test_parse_cpu_list_stride() {
        let cpuset = parse_cpu_list("0-10:2").unwrap();
        assert!(cpuset.is_set(0));
        assert!(!cpuset.is_set(1));
        assert!(cpuset.is_set(2));
        assert!(!cpuset.is_set(3));
        assert!(cpuset.is_set(4));
        assert!(cpuset.is_set(6));
        assert!(cpuset.is_set(8));
        assert!(cpuset.is_set(10));
        assert!(!cpuset.is_set(5));
    }

    #[test]
    fn test_parse_cpu_list_mixed() {
        let cpuset = parse_cpu_list("0,2-4,8").unwrap();
        assert!(cpuset.is_set(0));
        assert!(!cpuset.is_set(1));
        assert!(cpuset.is_set(2));
        assert!(cpuset.is_set(3));
        assert!(cpuset.is_set(4));
        assert!(!cpuset.is_set(5));
        assert!(cpuset.is_set(8));
    }

    #[test]
    fn test_format_hex_mask() {
        let mut cpuset = CpuSet::new();
        cpuset.set(0);
        cpuset.set(1);
        cpuset.set(2);
        cpuset.set(3);
        assert_eq!(format_hex_mask(&cpuset), "f");
    }

    #[test]
    fn test_format_hex_mask_multi_nibble() {
        let mut cpuset = CpuSet::new();
        cpuset.set(0);
        cpuset.set(4);
        assert_eq!(format_hex_mask(&cpuset), "11");
    }

    #[test]
    fn test_format_cpu_list_ranges() {
        let mut cpuset = CpuSet::new();
        cpuset.set(0);
        cpuset.set(2);
        cpuset.set(3);
        cpuset.set(4);
        cpuset.set(8);
        assert_eq!(format_cpu_list(&cpuset), "0,2-4,8");
    }

    #[test]
    fn test_format_cpu_list_single() {
        let mut cpuset = CpuSet::new();
        cpuset.set(5);
        assert_eq!(format_cpu_list(&cpuset), "5");
    }

    #[test]
    fn test_roundtrip_hex() {
        let original = parse_hex_mask("a5").unwrap();
        let formatted = format_hex_mask(&original);
        let roundtrip = parse_hex_mask(&formatted).unwrap();
        assert_eq!(original, roundtrip);
    }
}
