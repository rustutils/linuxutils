use clap::Parser;
use procfs::Current;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    process::ExitCode,
};

const CPU_SYS: &str = "/sys/devices/system/cpu";
const NODE_SYS: &str = "/sys/devices/system/node";

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

struct CacheInfo {
    name: String,
    size: String,
    instances: usize,
}

fn gather_caches() -> Vec<CacheInfo> {
    let mut seen: BTreeMap<String, (String, BTreeSet<String>)> =
        BTreeMap::new();

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
            let size = sysfs_read(&format!("{base}/size")).unwrap_or_default();
            let shared = sysfs_read(&format!("{base}/shared_cpu_list"))
                .unwrap_or_default();

            let type_abbr = match cache_type.as_str() {
                "Data" => "d",
                "Instruction" => "i",
                _ => "",
            };
            let name = format!("L{level}{type_abbr}");

            seen.entry(name)
                .or_insert_with(|| (size.clone(), BTreeSet::new()))
                .1
                .insert(shared);
        }
    }

    seen.into_iter()
        .map(|(name, (size, shared_sets))| CacheInfo {
            name,
            size,
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

fn print_field(label: &str, value: &str) {
    println!("{label:<23}{value}");
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

    // Basic fields from first CPU
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

    // Architecture
    let arch = std::env::consts::ARCH;
    let op_modes = match arch {
        "x86_64" => "32-bit, 64-bit",
        "aarch64" => "32-bit, 64-bit",
        "x86" => "32-bit",
        _ => arch,
    };

    // Online/offline
    let online = sysfs_read(&format!("{CPU_SYS}/online"))
        .unwrap_or_else(|| format!("0-{}", num_cpus - 1));
    let offline = sysfs_read(&format!("{CPU_SYS}/offline")).unwrap_or_default();

    // Topology: sockets, cores, threads
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

    // Frequency
    let max_mhz =
        sysfs_read(&format!("{CPU_SYS}/cpu0/cpufreq/cpuinfo_max_freq"))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|khz| khz / 1000.0);
    let min_mhz =
        sysfs_read(&format!("{CPU_SYS}/cpu0/cpufreq/cpuinfo_min_freq"))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|khz| khz / 1000.0);

    // Virtualization
    let virt = if flags_str.contains(" svm ")
        || flags_str.starts_with("svm ")
        || flags_str.ends_with(" svm")
    {
        Some("AMD-V")
    } else if flags_str.contains(" vmx ")
        || flags_str.starts_with("vmx ")
        || flags_str.ends_with(" vmx")
    {
        Some("VT-x")
    } else {
        None
    };

    // Output
    print_field("Architecture:", arch);
    print_field("CPU op-mode(s):", op_modes);
    if !addr_sizes.is_empty() {
        print_field("Address sizes:", &addr_sizes);
    }
    #[cfg(target_endian = "little")]
    print_field("Byte Order:", "Little Endian");
    #[cfg(target_endian = "big")]
    print_field("Byte Order:", "Big Endian");

    print_field("CPU(s):", &online_cpus.len().to_string());
    print_field("On-line CPU(s) list:", &online);
    if !offline.is_empty() {
        print_field("Off-line CPU(s) list:", &offline);
    }

    print_field("Vendor ID:", &vendor);
    print_field("Model name:", &model_name);
    print_field("CPU family:", &cpu_family);
    print_field("Model:", &model);
    print_field("Thread(s) per core:", &threads_per_core.to_string());
    print_field("Core(s) per socket:", &cores_per_sock.to_string());
    print_field("Socket(s):", &num_sockets.to_string());
    print_field("Stepping:", &stepping);

    if let Some(max) = max_mhz {
        print_field("CPU max MHz:", &format!("{max:.4}"));
    }
    if let Some(min) = min_mhz {
        print_field("CPU min MHz:", &format!("{min:.4}"));
    }
    print_field("BogoMIPS:", &bogomips);

    if let Some(v) = virt {
        print_field("Virtualization:", v);
    }

    // Caches
    let caches = gather_caches();
    for cache in &caches {
        let label = format!("{} cache:", cache.name);
        let value = if cache.instances > 1 {
            format!("{} ({} instances)", cache.size, cache.instances)
        } else {
            cache.size.clone()
        };
        print_field(&label, &value);
    }

    // NUMA
    let numa_nodes = gather_numa_nodes();
    if !numa_nodes.is_empty() {
        print_field("NUMA node(s):", &numa_nodes.len().to_string());
        for (node, cpulist) in &numa_nodes {
            print_field(&format!("NUMA node{node} CPU(s):"), cpulist);
        }
    }

    // Vulnerabilities
    let vulns = gather_vulnerabilities();
    for (name, status) in &vulns {
        let label = format!("Vulnerability {name}:");
        print_field(&label, status);
    }

    ExitCode::SUCCESS
}
