use clap::Parser;
use procfs::Current;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    process::ExitCode,
};

const CPU_SYS: &str = "/sys/devices/system/cpu";
const NODE_SYS: &str = "/sys/devices/system/node";
const FIELD_WIDTH: usize = 25;

/// Display information about the CPU architecture.
///
/// Gathers CPU information from /proc/cpuinfo and sysfs, displaying
/// architecture details, topology, caches, frequencies, and vulnerabilities.
#[derive(Parser)]
#[command(
    name = "lscpu",
    about = "Display information about the CPU architecture"
)]
pub struct Args {
    /// Print sizes in bytes
    #[arg(short = 'B', long)]
    bytes: bool,
}

fn sysfs_read(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn cpuinfo_field(
    cpuinfo: &procfs::CpuInfo,
    cpu: usize,
    field: &str,
) -> Option<String> {
    cpuinfo
        .get_info(cpu)
        .and_then(|info| info.get(field).map(|s| s.to_string()))
}

fn parse_cpu_list(s: &str) -> Vec<u32> {
    let mut result = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            if let (Ok(s), Ok(e)) = (start.parse::<u32>(), end.parse::<u32>()) {
                result.extend(s..=e);
            }
        } else if let Ok(n) = part.parse::<u32>() {
            result.push(n);
        }
    }
    result
}

fn parse_cache_size_kb(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(num) = s.strip_suffix('K') {
        num.trim().parse().ok()
    } else if let Some(num) = s.strip_suffix('M') {
        num.trim().parse::<u64>().ok().map(|n| n * 1024)
    } else {
        s.parse().ok()
    }
}

fn format_cache_size(total_kb: u64) -> String {
    if total_kb >= 1024 && total_kb.is_multiple_of(1024) {
        format!("{} MiB", total_kb / 1024)
    } else {
        format!("{total_kb} KiB")
    }
}

struct CacheInfo {
    name: String,
    one_size_kb: u64,
    instances: usize,
}

fn gather_caches() -> Vec<CacheInfo> {
    let mut seen: BTreeMap<String, (u64, BTreeSet<String>)> = BTreeMap::new();

    let Ok(entries) = fs::read_dir(CPU_SYS) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        let cpu_name = entry.file_name();
        let cpu_str = cpu_name.to_string_lossy();
        if !cpu_str.starts_with("cpu")
            || cpu_str[3..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_ascii_digit())
        {
            continue;
        }

        let cache_dir = format!("{CPU_SYS}/{cpu_str}/cache");
        let Ok(cache_entries) = fs::read_dir(&cache_dir) else {
            continue;
        };

        for ce in cache_entries.flatten() {
            let idx_name = ce.file_name();
            let idx_str = idx_name.to_string_lossy();
            if !idx_str.starts_with("index") {
                continue;
            }

            let base = format!("{cache_dir}/{idx_str}");
            let level =
                sysfs_read(&format!("{base}/level")).unwrap_or_default();
            let cache_type =
                sysfs_read(&format!("{base}/type")).unwrap_or_default();
            let size_str =
                sysfs_read(&format!("{base}/size")).unwrap_or_default();
            let shared = sysfs_read(&format!("{base}/shared_cpu_list"))
                .unwrap_or_default();

            let size_kb = parse_cache_size_kb(&size_str).unwrap_or(0);
            let type_abbr = match cache_type.as_str() {
                "Data" => "d",
                "Instruction" => "i",
                _ => "",
            };
            let name = format!("L{level}{type_abbr}");

            seen.entry(name)
                .or_insert_with(|| (size_kb, BTreeSet::new()))
                .1
                .insert(shared);
        }
    }

    seen.into_iter()
        .map(|(name, (one_size_kb, shared_sets))| CacheInfo {
            name,
            one_size_kb,
            instances: shared_sets.len(),
        })
        .collect()
}

fn gather_numa_nodes() -> Vec<(u32, String)> {
    let Ok(entries) = fs::read_dir(NODE_SYS) else {
        return Vec::new();
    };

    let mut nodes: Vec<(u32, String)> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if let Some(num_str) = name_str.strip_prefix("node")
            && let Ok(num) = num_str.parse::<u32>()
        {
            let cpulist = sysfs_read(&format!("{NODE_SYS}/{name_str}/cpulist"))
                .unwrap_or_default();
            nodes.push((num, cpulist));
        }
    }
    nodes.sort_by_key(|(n, _)| *n);
    nodes
}

fn gather_vulnerabilities() -> Vec<(String, String)> {
    let vuln_dir = format!("{CPU_SYS}/vulnerabilities");
    let Ok(entries) = fs::read_dir(&vuln_dir) else {
        return Vec::new();
    };

    let mut vulns: Vec<(String, String)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let status = sysfs_read(&format!("{vuln_dir}/{name}"))?;
            Some((name, status))
        })
        .collect();

    vulns.sort_by(|(a, _), (b, _)| a.cmp(b));
    vulns
}

fn pretty_vuln_name(file_name: &str) -> String {
    let mut result = file_name.replace('_', " ");
    if let Some(first) = result.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    result
}

fn field(indent: usize, label: &str, value: &str) {
    let padding = " ".repeat(indent);
    let total_label_width = FIELD_WIDTH - indent;
    println!("{padding}{label:<total_label_width$}{value}");
}

fn wrap_flags(flags: &str, indent: usize) {
    let padding = " ".repeat(indent);
    let total_label_width = FIELD_WIDTH - indent;
    let label = "Flags:";
    let content_start = FIELD_WIDTH;
    let wrap_width = 80usize.saturating_sub(content_start);

    let words: Vec<&str> = flags.split_whitespace().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for word in &words {
        if !current_line.is_empty()
            && current_line.len() + 1 + word.len() > wrap_width
        {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            println!("{padding}{label:<total_label_width$}{line}");
        } else {
            println!("{:>content_start$}{line}", "");
        }
    }
}

pub fn run(_args: Args) -> ExitCode {
    let cpuinfo = match procfs::CpuInfo::current() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("lscpu: failed to read /proc/cpuinfo: {e}");
            return ExitCode::FAILURE;
        }
    };

    let num_cpus = cpuinfo.num_cores();

    let vendor = cpuinfo_field(&cpuinfo, 0, "vendor_id").unwrap_or_default();
    let model_name =
        cpuinfo_field(&cpuinfo, 0, "model name").unwrap_or_default();
    let cpu_family =
        cpuinfo_field(&cpuinfo, 0, "cpu family").unwrap_or_default();
    let model = cpuinfo_field(&cpuinfo, 0, "model").unwrap_or_default();
    let stepping = cpuinfo_field(&cpuinfo, 0, "stepping").unwrap_or_default();
    let bogomips = cpuinfo_field(&cpuinfo, 0, "bogomips").unwrap_or_default();
    let flags_str = cpuinfo_field(&cpuinfo, 0, "flags").unwrap_or_default();
    let addr_sizes =
        cpuinfo_field(&cpuinfo, 0, "address sizes").unwrap_or_default();

    let arch = std::env::consts::ARCH;
    let op_modes = match arch {
        "x86_64" => "32-bit, 64-bit",
        "aarch64" => "32-bit, 64-bit",
        "x86" => "32-bit",
        _ => arch,
    };

    let online = sysfs_read(&format!("{CPU_SYS}/online"))
        .unwrap_or_else(|| format!("0-{}", num_cpus - 1));
    let offline = sysfs_read(&format!("{CPU_SYS}/offline")).unwrap_or_default();

    let mut sockets: BTreeSet<u32> = BTreeSet::new();
    let mut cores_per_socket: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    let online_cpus = parse_cpu_list(&online);

    for &cpu in &online_cpus {
        let pkg = sysfs_read(&format!(
            "{CPU_SYS}/cpu{cpu}/topology/physical_package_id"
        ))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
        let core = sysfs_read(&format!("{CPU_SYS}/cpu{cpu}/topology/core_id"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        sockets.insert(pkg);
        cores_per_socket.entry(pkg).or_default().insert(core);
    }

    let num_sockets = sockets.len().max(1);
    let cores_per_sock = if !cores_per_socket.is_empty() {
        cores_per_socket.values().next().unwrap().len()
    } else {
        num_cpus
    };
    let threads_per_core = if cores_per_sock > 0 && num_sockets > 0 {
        online_cpus.len() / (num_sockets * cores_per_sock)
    } else {
        1
    }
    .max(1);

    let max_mhz =
        sysfs_read(&format!("{CPU_SYS}/cpu0/cpufreq/cpuinfo_max_freq"))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|khz| khz / 1000.0);
    let min_mhz =
        sysfs_read(&format!("{CPU_SYS}/cpu0/cpufreq/cpuinfo_min_freq"))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|khz| khz / 1000.0);

    let virt = if flags_str.split_whitespace().any(|f| f == "svm") {
        Some("AMD-V")
    } else if flags_str.split_whitespace().any(|f| f == "vmx") {
        Some("VT-x")
    } else {
        None
    };

    // Architecture section
    field(0, "Architecture:", arch);
    field(2, "CPU op-mode(s):", op_modes);
    if !addr_sizes.is_empty() {
        field(2, "Address sizes:", &addr_sizes);
    }
    #[cfg(target_endian = "little")]
    field(2, "Byte Order:", "Little Endian");
    #[cfg(target_endian = "big")]
    field(2, "Byte Order:", "Big Endian");

    // CPU count
    field(0, "CPU(s):", &online_cpus.len().to_string());
    field(2, "On-line CPU(s) list:", &online);
    if !offline.is_empty() {
        field(2, "Off-line CPU(s) list:", &offline);
    }

    // Vendor / Model
    field(0, "Vendor ID:", &vendor);
    field(2, "Model name:", &model_name);
    field(4, "CPU family:", &cpu_family);
    field(4, "Model:", &model);
    field(4, "Thread(s) per core:", &threads_per_core.to_string());
    field(4, "Core(s) per socket:", &cores_per_sock.to_string());
    field(4, "Socket(s):", &num_sockets.to_string());
    field(4, "Stepping:", &stepping);
    if let Some(max) = max_mhz {
        field(4, "CPU max MHz:", &format!("{max:.4}"));
    }
    if let Some(min) = min_mhz {
        field(4, "CPU min MHz:", &format!("{min:.4}"));
    }
    field(4, "BogoMIPS:", &bogomips);
    if !flags_str.is_empty() {
        wrap_flags(&flags_str, 4);
    }

    // Virtualization
    if let Some(v) = virt {
        println!("Virtualization features:");
        field(2, "Virtualization:", v);
    }

    // Caches
    let caches = gather_caches();
    if !caches.is_empty() {
        println!("Caches (sum of all):");
        for cache in &caches {
            let total_kb = cache.one_size_kb * cache.instances as u64;
            let size_str = format_cache_size(total_kb);
            let value = if cache.instances > 1 {
                format!("{size_str} ({} instances)", cache.instances)
            } else {
                format!("{size_str} (1 instance)")
            };
            field(2, &format!("{}:", cache.name), &value);
        }
    }

    // NUMA
    let numa_nodes = gather_numa_nodes();
    if !numa_nodes.is_empty() {
        println!("NUMA:");
        field(2, "NUMA node(s):", &numa_nodes.len().to_string());
        for (node, cpulist) in &numa_nodes {
            field(2, &format!("NUMA node{node} CPU(s):"), cpulist);
        }
    }

    // Vulnerabilities
    let vulns = gather_vulnerabilities();
    if !vulns.is_empty() {
        println!("Vulnerabilities:");
        for (name, status) in &vulns {
            let pretty = pretty_vuln_name(name);
            field(2, &format!("{pretty}:"), status);
        }
    }

    ExitCode::SUCCESS
}
