use clap::Parser;
use cols::{OutputMode, Table, WidthHint, print_table};
use std::{
    io::{self, BufRead},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Parser)]
#[command(about = "Utility to display kernel interrupt information")]
pub struct Args {
    /// Use JSON output format
    #[arg(short = 'J', long)]
    json: bool,

    /// Use key="value" output format
    #[arg(short = 'P', long)]
    pairs: bool,

    /// Don't print headings
    #[arg(short = 'n', long)]
    noheadings: bool,

    /// Output columns to print
    #[arg(short, long, value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Sort by column name
    #[arg(short, long)]
    sort: Option<String>,

    /// Show softirqs instead of interrupts
    #[arg(short = 'S', long)]
    softirq: bool,

    /// Only IRQs with counters above this threshold
    #[arg(short, long)]
    threshold: Option<String>,

    /// Only show counters for these CPUs
    #[arg(short = 'C', long, value_delimiter = ',')]
    cpu_list: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Col {
    Irq,
    Total,
    Name,
}

impl Col {
    fn name(self) -> &'static str {
        match self {
            Col::Irq => "IRQ",
            Col::Total => "TOTAL",
            Col::Name => "NAME",
        }
    }

    fn whint(self) -> WidthHint {
        match self {
            Col::Irq => WidthHint::Auto,
            Col::Total => WidthHint::Auto,
            Col::Name => WidthHint::Auto,
        }
    }

    fn is_right(self) -> bool {
        matches!(self, Col::Irq | Col::Total)
    }

    fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "IRQ" => Some(Col::Irq),
            "TOTAL" => Some(Col::Total),
            "NAME" => Some(Col::Name),
            _ => None,
        }
    }
}

const DEFAULT_COLUMNS: &[Col] = &[Col::Irq, Col::Total, Col::Name];

#[derive(Debug)]
struct IrqEntry {
    irq: String,
    total: u64,
    name: String,
}

fn parse_threshold(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_part, suffix) = if s.ends_with("KiB") || s.ends_with("K") {
        (s.trim_end_matches("KiB").trim_end_matches('K'), 1000u64)
    } else if s.ends_with("MiB") || s.ends_with("M") {
        (s.trim_end_matches("MiB").trim_end_matches('M'), 1_000_000)
    } else if s.ends_with("GiB") || s.ends_with("G") {
        (
            s.trim_end_matches("GiB").trim_end_matches('G'),
            1_000_000_000,
        )
    } else {
        (s, 1)
    };

    let val: f64 = num_part.trim().parse().ok()?;
    Some((val * suffix as f64) as u64)
}

fn parse_cpu_list(list: &[String]) -> Vec<usize> {
    let mut cpus = Vec::new();
    for item in list {
        for part in item.split(',') {
            let part = part.trim();
            if let Some((start, end)) = part.split_once('-') {
                if let (Ok(s), Ok(e)) =
                    (start.trim().parse::<usize>(), end.trim().parse::<usize>())
                {
                    cpus.extend(s..=e);
                }
            } else if let Ok(n) = part.parse::<usize>() {
                cpus.push(n);
            }
        }
    }
    cpus.sort();
    cpus.dedup();
    cpus
}

fn read_interrupts(
    softirq: bool,
    cpu_filter: &Option<Vec<usize>>,
) -> Result<Vec<IrqEntry>, io::Error> {
    let path = if softirq {
        PathBuf::from("/proc/softirqs")
    } else {
        PathBuf::from("/proc/interrupts")
    };

    let file = std::fs::File::open(&path)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();

    // First line is the CPU header — parse to get CPU count.
    let header = lines.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "empty interrupts file")
    })??;
    let _num_cpus = header.split_whitespace().count();

    let mut entries = Vec::new();
    for line in lines {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Format: "IRQ: count0 count1 ... countN  description"
        // or for named IRQs without colon: "NAME count0 count1 ... countN"
        let (irq_name, rest) = if let Some((name, rest)) = line.split_once(':')
        {
            (name.trim().to_string(), rest.trim())
        } else {
            continue;
        };

        let parts: Vec<&str> = rest.split_whitespace().collect();

        // Parse per-CPU counts. Some IRQs (like ERR, MIS) have fewer
        // counters than CPUs.
        let mut per_cpu: Vec<u64> = Vec::new();
        let mut desc_start = 0;
        for (i, part) in parts.iter().enumerate() {
            if let Ok(n) = part.parse::<u64>() {
                per_cpu.push(n);
            } else {
                desc_start = i;
                break;
            }
            desc_start = i + 1;
        }

        let total = if let Some(cpus) = cpu_filter {
            cpus.iter().filter_map(|&c| per_cpu.get(c)).sum()
        } else {
            per_cpu.iter().sum()
        };

        let name = parts[desc_start..].join(" ");

        entries.push(IrqEntry {
            irq: irq_name,
            total,
            name,
        });
    }

    Ok(entries)
}

pub fn run(args: Args) -> ExitCode {
    let cpu_filter = args.cpu_list.as_ref().map(|l| parse_cpu_list(l));

    let mut entries = match read_interrupts(args.softirq, &cpu_filter) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("lsirq: failed to read interrupts: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Apply threshold filter.
    if let Some(ref threshold_str) = args.threshold {
        if let Some(threshold) = parse_threshold(threshold_str) {
            entries.retain(|e| e.total >= threshold);
        } else {
            eprintln!("lsirq: invalid threshold: {threshold_str}");
            return ExitCode::FAILURE;
        }
    }

    let columns = if let Some(ref names) = args.output {
        let mut cols = Vec::new();
        for name in names {
            match Col::from_name(name.trim()) {
                Some(c) => cols.push(c),
                None => {
                    eprintln!("lsirq: unknown column: {name}");
                    return ExitCode::FAILURE;
                }
            }
        }
        cols
    } else {
        DEFAULT_COLUMNS.to_vec()
    };

    // Sort entries. Default is by TOTAL descending.
    let sort_col = args
        .sort
        .as_ref()
        .and_then(|s| Col::from_name(s))
        .unwrap_or(Col::Total);

    match sort_col {
        Col::Irq => entries.sort_by(|a, b| a.irq.cmp(&b.irq)),
        Col::Total => entries.sort_by(|a, b| b.total.cmp(&a.total)),
        Col::Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    let mut table = Table::new();
    table.set_name("interrupts");

    if args.json {
        table.set_output_mode(OutputMode::Json);
    } else if args.pairs {
        table.set_output_mode(OutputMode::Export);
    }

    if args.noheadings {
        table.enable_no_headings(true);
    }

    for col in &columns {
        let idx = table.new_column(col.name());
        table.column_mut(idx).unwrap().set_width_hint(col.whint());
        if col.is_right() {
            table.column_mut(idx).unwrap().set_right(true);
        }
    }

    for entry in &entries {
        let line_id = table.new_line(None);
        let line = table.line_mut(line_id);

        for (ci, col) in columns.iter().enumerate() {
            let val = match col {
                Col::Irq => entry.irq.clone(),
                Col::Total => entry.total.to_string(),
                Col::Name => entry.name.clone(),
            };
            line.set_data(ci, &val);
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = print_table(&table, &mut out) {
        eprintln!("lsirq: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
