use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use cols::{OutputMode, Table, WidthHint, print_table};
use crate::sysfs::SysfsDevice;
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "lsmem",
    about = "List the ranges of available memory with their online status"
)]
pub struct Args {
    /// List each individual memory block
    #[arg(short, long)]
    all: bool,

    /// Print SIZE in bytes rather than human-readable format
    #[arg(short, long)]
    bytes: bool,

    /// Use JSON output format
    #[arg(short = 'J', long)]
    json: bool,

    /// Don't print headings
    #[arg(short = 'n', long)]
    noheadings: bool,

    /// Output columns to print
    #[arg(short, long, value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Output all available columns
    #[arg(long)]
    output_all: bool,

    /// Use key="value" output format
    #[arg(short = 'P', long)]
    pairs: bool,

    /// Use raw output format
    #[arg(short, long)]
    raw: bool,

    /// Split ranges by specified columns
    #[arg(short = 'S', long, value_delimiter = ',')]
    split: Option<Vec<String>>,

    /// Use specified directory as system root
    #[arg(short, long)]
    sysroot: Option<PathBuf>,

    /// Print summary information (never, always, only)
    #[arg(long, default_value = "always")]
    summary: SummaryMode,
}

#[derive(Clone, Debug)]
enum SummaryMode {
    Never,
    Always,
    Only,
}

impl std::str::FromStr for SummaryMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "never" => Ok(Self::Never),
            "always" => Ok(Self::Always),
            "only" => Ok(Self::Only),
            _ => Err(format!("invalid summary mode: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Col {
    Range,
    Size,
    State,
    Removable,
    Block,
    Node,
    Zones,
}

impl Col {
    fn name(self) -> &'static str {
        match self {
            Col::Range => "RANGE",
            Col::Size => "SIZE",
            Col::State => "STATE",
            Col::Removable => "REMOVABLE",
            Col::Block => "BLOCK",
            Col::Node => "NODE",
            Col::Zones => "ZONES",
        }
    }

    fn whint(self) -> WidthHint {
        match self {
            Col::Range => WidthHint::Auto,
            Col::Size => WidthHint::Fixed(5),
            Col::State => WidthHint::Fixed(6),
            Col::Removable => WidthHint::Fixed(9),
            Col::Block => WidthHint::Auto,
            Col::Node => WidthHint::Fixed(4),
            Col::Zones => WidthHint::Auto,
        }
    }

    fn is_right(self) -> bool {
        matches!(self, Col::Size | Col::Removable | Col::Block)
    }

    fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "RANGE" => Some(Col::Range),
            "SIZE" => Some(Col::Size),
            "STATE" => Some(Col::State),
            "REMOVABLE" => Some(Col::Removable),
            "BLOCK" => Some(Col::Block),
            "NODE" => Some(Col::Node),
            "ZONES" => Some(Col::Zones),
            _ => None,
        }
    }
}

const DEFAULT_COLUMNS: &[Col] = &[
    Col::Range,
    Col::Size,
    Col::State,
    Col::Removable,
    Col::Block,
];
const ALL_COLUMNS: &[Col] = &[
    Col::Range,
    Col::Size,
    Col::State,
    Col::Removable,
    Col::Block,
    Col::Node,
    Col::Zones,
];

/// Attributes that control how memory blocks are split into ranges.
const DEFAULT_SPLIT: &[Col] = &[Col::State, Col::Removable, Col::Node];

#[derive(Debug)]
struct MemBlock {
    index: u64,
    state: String,
    removable: bool,
    node: Option<u64>,
    zones: String,
    block_size: u64,
}

impl MemBlock {
    fn start_addr(&self) -> u64 {
        self.index * self.block_size
    }

    fn end_addr(&self) -> u64 {
        (self.index + 1) * self.block_size - 1
    }
}

#[derive(Debug)]
struct MemRange {
    start_block: u64,
    end_block: u64,
    start_addr: u64,
    end_addr: u64,
    size: u64,
    state: String,
    removable: bool,
    node: Option<u64>,
    zones: String,
}

fn read_memory_blocks(sysroot: &Path) -> Result<(u64, Vec<MemBlock>), String> {
    let mem_dir = SysfsDevice::new(sysroot.join("sys/devices/system/memory"));

    let block_size = mem_dir
        .read_attr_hex("block_size_bytes")
        .map_err(|e| format!("failed to read block_size_bytes: {e}"))?;

    let children = mem_dir
        .children_with_prefix("memory")
        .map_err(|e| format!("failed to list memory blocks: {e}"))?;

    let mut blocks = Vec::new();
    for child in children {
        let index = child
            .read_attr_hex("phys_index")
            .map_err(|e| format!("failed to read phys_index: {e}"))?;
        let state = child
            .read_attr("state")
            .map_err(|e| format!("failed to read state: {e}"))?;
        let removable = child.read_attr_bool("removable").unwrap_or(false);
        let node = find_node(&child);
        let zones = child.read_attr("valid_zones").unwrap_or_default();
        // Only the first zone matters for display.
        let zones = zones.split_whitespace().next().unwrap_or("").to_string();

        blocks.push(MemBlock {
            index,
            state,
            removable,
            node,
            zones,
            block_size,
        });
    }

    blocks.sort_by_key(|b| b.index);
    Ok((block_size, blocks))
}

fn find_node(dev: &SysfsDevice) -> Option<u64> {
    // NUMA node is in a symlink like /sys/devices/system/memory/memoryN/nodeN
    let path = dev.path();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && let Some(n) = name.strip_prefix("node")
                && let Ok(num) = n.parse::<u64>()
            {
                return Some(num);
            }
        }
    }
    None
}

fn merge_blocks(blocks: &[MemBlock], split_cols: &[Col]) -> Vec<MemRange> {
    if blocks.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut start = 0;

    for i in 1..blocks.len() {
        let should_split = blocks[i].index != blocks[i - 1].index + 1
            || split_cols.iter().any(|col| match col {
                Col::State => blocks[i].state != blocks[start].state,
                Col::Removable => {
                    blocks[i].removable != blocks[start].removable
                }
                Col::Node => blocks[i].node != blocks[start].node,
                Col::Zones => blocks[i].zones != blocks[start].zones,
                _ => false,
            });

        if should_split {
            ranges.push(make_range(&blocks[start..i]));
            start = i;
        }
    }
    ranges.push(make_range(&blocks[start..]));
    ranges
}

fn make_range(blocks: &[MemBlock]) -> MemRange {
    let first = &blocks[0];
    let last = &blocks[blocks.len() - 1];
    MemRange {
        start_block: first.index,
        end_block: last.index,
        start_addr: first.start_addr(),
        end_addr: last.end_addr(),
        size: (last.index - first.index + 1) * first.block_size,
        state: first.state.clone(),
        removable: first.removable,
        node: first.node,
        zones: first.zones.clone(),
    }
}

fn format_size(bytes: u64, human: bool) -> String {
    if !human {
        return bytes.to_string();
    }
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P", "E"];
    let mut val = bytes as f64;
    for unit in UNITS {
        if val < 1024.0 || *unit == "E" {
            if (val - val.round()).abs() < 0.05 {
                return format!("{}{unit}", val.round() as u64);
            }
            // Use C-style rounding (half-up) instead of Rust's default
            // banker's rounding: add a tiny epsilon before formatting.
            let rounded = (val * 10.0 + 0.5).floor() / 10.0;
            return format!("{rounded:.1}{unit}");
        }
        val /= 1024.0;
    }
    unreachable!()
}

pub fn run(args: Args) -> ExitCode {
    let sysroot = args.sysroot.as_deref().unwrap_or(Path::new("/"));

    let (block_size, blocks) = match read_memory_blocks(sysroot) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("lsmem: {e}");
            return ExitCode::FAILURE;
        }
    };

    let columns = if args.output_all {
        ALL_COLUMNS.to_vec()
    } else if let Some(ref names) = args.output {
        let mut cols = Vec::new();
        for name in names {
            let name = name.trim();
            match Col::from_name(name) {
                Some(c) => cols.push(c),
                None => {
                    eprintln!("lsmem: unknown column: {name}");
                    return ExitCode::FAILURE;
                }
            }
        }
        cols
    } else {
        DEFAULT_COLUMNS.to_vec()
    };

    let split_cols = if let Some(ref names) = args.split {
        if names.len() == 1 && names[0].eq_ignore_ascii_case("none") {
            Vec::new()
        } else {
            names
                .iter()
                .filter_map(|n| Col::from_name(n.trim()))
                .collect()
        }
    } else {
        DEFAULT_SPLIT.to_vec()
    };

    let ranges = if args.all {
        // --all: each block is its own range
        blocks
            .iter()
            .map(|b| MemRange {
                start_block: b.index,
                end_block: b.index,
                start_addr: b.start_addr(),
                end_addr: b.end_addr(),
                size: b.block_size,
                state: b.state.clone(),
                removable: b.removable,
                node: b.node,
                zones: b.zones.clone(),
            })
            .collect()
    } else {
        merge_blocks(&blocks, &split_cols)
    };

    let human = !args.bytes;
    let show_summary = match args.summary {
        SummaryMode::Never => false,
        SummaryMode::Only => true,
        SummaryMode::Always => !args.raw && !args.pairs && !args.json,
    };
    let show_table = !matches!(args.summary, SummaryMode::Only);

    if show_table {
        let mut table = Table::new();
        table.name_set("memory");

        if args.json {
            table.output_mode_set(OutputMode::Json);
        } else if args.pairs {
            table.output_mode_set(OutputMode::Export);
        } else if args.raw {
            table.output_mode_set(OutputMode::Raw);
        }

        if args.noheadings {
            table.headings_set(false);
        }

        for col in &columns {
            let idx = table.new_column(col.name());
            table.column_mut(idx).unwrap().width_hint_set(col.whint());
            if col.is_right() {
                table.column_mut(idx).unwrap().right_set(true);
            }
        }

        for range in &ranges {
            let line_id = table.new_line(None);
            let line = table.line_mut(line_id);

            for (ci, col) in columns.iter().enumerate() {
                let val = match col {
                    Col::Range => format!(
                        "0x{:016x}-0x{:016x}",
                        range.start_addr, range.end_addr
                    ),
                    Col::Size => format_size(range.size, human),
                    Col::State => range.state.clone(),
                    Col::Removable => {
                        if range.removable { "yes" } else { "no" }.to_string()
                    }
                    Col::Block => {
                        if range.start_block == range.end_block {
                            range.start_block.to_string()
                        } else {
                            format!("{}-{}", range.start_block, range.end_block)
                        }
                    }
                    Col::Node => {
                        range.node.map_or(String::new(), |n| n.to_string())
                    }
                    Col::Zones => range.zones.clone(),
                };
                line.data_set(ci, &val);
            }
        }

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        if let Err(e) = print_table(&table, &mut out) {
            eprintln!("lsmem: {e}");
            return ExitCode::FAILURE;
        }
    }

    if show_summary {
        let total_online: u64 = blocks
            .iter()
            .filter(|b| b.state == "online")
            .map(|b| b.block_size)
            .sum();
        let total_offline: u64 = blocks
            .iter()
            .filter(|b| b.state != "online")
            .map(|b| b.block_size)
            .sum();

        if show_table {
            println!();
        }
        let w = 38;
        println!(
            "Memory block size:{:>pad$}",
            format_size(block_size, human),
            pad = w - 18,
        );
        println!(
            "Total online memory:{:>pad$}",
            format_size(total_online, human),
            pad = w - 20,
        );
        println!(
            "Total offline memory:{:>pad$}",
            format_size(total_offline, human),
            pad = w - 21,
        );
    }

    ExitCode::SUCCESS
}
