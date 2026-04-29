use clap::{Parser, Subcommand};
use cols::{OutputMode, Table, WidthHint, print_table};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    process::ExitCode,
};

// rfkill type constants (from linux/rfkill.h)
const TYPE_ALL: u8 = 0;
const TYPE_WLAN: u8 = 1;
const TYPE_BLUETOOTH: u8 = 2;
const TYPE_UWB: u8 = 3;
const TYPE_WIMAX: u8 = 4;
const TYPE_WWAN: u8 = 5;
const TYPE_GPS: u8 = 6;
const TYPE_FM: u8 = 7;
const TYPE_NFC: u8 = 8;

// rfkill operation constants (from linux/rfkill.h)
const OP_ADD: u8 = 0;
const OP_DEL: u8 = 1;
const OP_CHANGE: u8 = 2;
const OP_CHANGE_ALL: u8 = 3;

#[derive(Parser)]
#[command(name = "rfkill", about = "Enable and disable wireless devices")]
pub struct Args {
    /// Use JSON output format
    #[arg(short = 'J', long)]
    json: bool,

    /// Don't print headings
    #[arg(short = 'n', long)]
    noheadings: bool,

    /// Define which output columns to use
    #[arg(short = 'o', long, value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Output all available columns
    #[arg(long)]
    output_all: bool,

    /// Use the raw output format
    #[arg(short = 'r', long)]
    raw: bool,

    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Listen for rfkill events and display them on stdout
    Event,

    /// List the current state of all available devices (deprecated format)
    List {
        /// Device identifiers (IDs or type names)
        #[arg()]
        identifiers: Vec<String>,
    },

    /// Disable the corresponding device
    Block {
        /// Device identifiers (IDs or type names)
        #[arg(required = true)]
        identifiers: Vec<String>,
    },

    /// Enable the corresponding device
    Unblock {
        /// Device identifiers (IDs or type names)
        #[arg(required = true)]
        identifiers: Vec<String>,
    },

    /// Enable or disable the corresponding device
    Toggle {
        /// Device identifiers (IDs or type names)
        #[arg(required = true)]
        identifiers: Vec<String>,
    },
}

#[derive(Debug)]
struct Device {
    id: u32,
    rf_type: u8,
    name: String,
    soft: bool,
    hard: bool,
}

impl Device {
    fn type_name(&self) -> &'static str {
        type_to_name(self.rf_type)
    }

    fn type_desc(&self) -> &'static str {
        type_to_desc(self.rf_type)
    }
}

fn type_to_name(t: u8) -> &'static str {
    match t {
        TYPE_WLAN => "wlan",
        TYPE_BLUETOOTH => "bluetooth",
        TYPE_UWB => "uwb",
        TYPE_WIMAX => "wimax",
        TYPE_WWAN => "wwan",
        TYPE_GPS => "gps",
        TYPE_FM => "fm",
        TYPE_NFC => "nfc",
        _ => "unknown",
    }
}

fn type_to_desc(t: u8) -> &'static str {
    match t {
        TYPE_WLAN => "Wireless LAN",
        TYPE_BLUETOOTH => "Bluetooth",
        TYPE_UWB => "Ultra-Wideband",
        TYPE_WIMAX => "WiMAX",
        TYPE_WWAN => "Wireless WAN",
        TYPE_GPS => "GPS",
        TYPE_FM => "FM",
        TYPE_NFC => "NFC",
        _ => "Unknown",
    }
}

fn name_to_type(s: &str) -> Option<u8> {
    match s {
        "all" => Some(TYPE_ALL),
        "wlan" | "wifi" => Some(TYPE_WLAN),
        "bluetooth" => Some(TYPE_BLUETOOTH),
        "uwb" | "ultrawideband" => Some(TYPE_UWB),
        "wimax" => Some(TYPE_WIMAX),
        "wwan" => Some(TYPE_WWAN),
        "gps" => Some(TYPE_GPS),
        "fm" => Some(TYPE_FM),
        "nfc" => Some(TYPE_NFC),
        _ => None,
    }
}

enum Identifier {
    Id(u32),
    Type(u8),
}

fn parse_identifier(s: &str) -> Option<Identifier> {
    if let Ok(id) = s.parse::<u32>() {
        return Some(Identifier::Id(id));
    }
    name_to_type(s).map(Identifier::Type)
}

fn read_devices() -> Vec<Device> {
    let mut devices = Vec::new();

    let entries = match fs::read_dir("/sys/class/rfkill") {
        Ok(e) => e,
        Err(_) => return devices,
    };

    for entry in entries.flatten() {
        let fname = entry.file_name();
        let fname_str = match fname.to_str() {
            Some(s) => s,
            None => continue,
        };

        let id: u32 = match fname_str.strip_prefix("rfkill") {
            Some(n) => match n.parse() {
                Ok(v) => v,
                Err(_) => continue,
            },
            None => continue,
        };

        let base = entry.path();

        let type_str = sysfs_read_str(&base.join("type")).unwrap_or_default();
        let dev_name = sysfs_read_str(&base.join("name")).unwrap_or_default();
        let soft = sysfs_read_u8(&base.join("soft")).unwrap_or(0) != 0;
        let hard = sysfs_read_u8(&base.join("hard")).unwrap_or(0) != 0;

        let rf_type = name_to_type(&type_str).unwrap_or(0);

        devices.push(Device {
            id,
            rf_type,
            name: dev_name,
            soft,
            hard,
        });
    }

    devices.sort_by_key(|d| d.id);
    devices
}

fn sysfs_read_str(path: &std::path::Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn sysfs_read_u8(path: &std::path::Path) -> Option<u8> {
    sysfs_read_str(path)?.parse().ok()
}

// Write a packed rfkill_event (8 bytes) to /dev/rfkill.
// Layout: u32 idx, u8 type, u8 op, u8 soft, u8 hard (all in native byte order).
fn write_event(idx: u32, rf_type: u8, op: u8, soft: u8) -> io::Result<()> {
    let mut f = OpenOptions::new().write(true).open("/dev/rfkill")?;
    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&idx.to_ne_bytes());
    buf[4] = rf_type;
    buf[5] = op;
    buf[6] = soft;
    buf[7] = 0;
    f.write_all(&buf)
}

fn device_matches(dev: &Device, ident: &Identifier) -> bool {
    match ident {
        Identifier::Id(id) => dev.id == *id,
        Identifier::Type(TYPE_ALL) => true,
        Identifier::Type(t) => dev.rf_type == *t,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Col {
    Id,
    Type,
    Device,
    TypeDesc,
    Soft,
    Hard,
}

impl Col {
    fn name(self) -> &'static str {
        match self {
            Col::Id => "ID",
            Col::Type => "TYPE",
            Col::Device => "DEVICE",
            Col::TypeDesc => "TYPE-DESC",
            Col::Soft => "SOFT",
            Col::Hard => "HARD",
        }
    }

    fn whint(self) -> WidthHint {
        match self {
            Col::Id => WidthHint::Fixed(2),
            Col::Type => WidthHint::Fixed(9),
            Col::Device => WidthHint::Fixed(10),
            Col::TypeDesc => WidthHint::Fixed(12),
            Col::Soft => WidthHint::Fixed(9),
            Col::Hard => WidthHint::Fixed(9),
        }
    }

    fn is_right(self) -> bool {
        matches!(self, Col::Id)
    }

    fn from_name(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ID" => Some(Col::Id),
            "TYPE" => Some(Col::Type),
            "DEVICE" => Some(Col::Device),
            "TYPE-DESC" => Some(Col::TypeDesc),
            "SOFT" => Some(Col::Soft),
            "HARD" => Some(Col::Hard),
            _ => None,
        }
    }

    fn cell_value(self, dev: &Device) -> String {
        match self {
            Col::Id => dev.id.to_string(),
            Col::Type => dev.type_name().to_string(),
            Col::Device => dev.name.clone(),
            Col::TypeDesc => dev.type_desc().to_string(),
            Col::Soft => blocked_str(dev.soft).to_string(),
            Col::Hard => blocked_str(dev.hard).to_string(),
        }
    }
}

const DEFAULT_COLUMNS: &[Col] =
    &[Col::Id, Col::Type, Col::Device, Col::Soft, Col::Hard];

const ALL_COLUMNS: &[Col] = &[
    Col::Id,
    Col::Type,
    Col::Device,
    Col::TypeDesc,
    Col::Soft,
    Col::Hard,
];

fn blocked_str(blocked: bool) -> &'static str {
    if blocked { "blocked" } else { "unblocked" }
}

fn print_table_output(args: &Args, devices: &[Device]) -> ExitCode {
    let columns = if args.output_all {
        ALL_COLUMNS.to_vec()
    } else if let Some(ref names) = args.output {
        let mut cols = Vec::new();
        for name in names {
            match Col::from_name(name.trim()) {
                Some(c) => cols.push(c),
                None => {
                    eprintln!("rfkill: unknown column: {name}");
                    return ExitCode::FAILURE;
                }
            }
        }
        cols
    } else {
        DEFAULT_COLUMNS.to_vec()
    };

    let mut table = Table::new();
    table.name_set("rfkilldevs");

    if args.json {
        table.output_mode_set(OutputMode::Json);
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

    for dev in devices {
        let line_id = table.new_line(None);
        let line = table.line_mut(line_id);
        for (ci, col) in columns.iter().enumerate() {
            line.data_set(ci, &col.cell_value(dev));
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = print_table(&table, &mut out) {
        eprintln!("rfkill: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn print_legacy_list(devices: &[Device]) {
    for dev in devices {
        println!("{}: {}: {}", dev.id, dev.name, dev.type_desc());
        println!("\tSoft blocked: {}", if dev.soft { "yes" } else { "no" });
        println!("\tHard blocked: {}", if dev.hard { "yes" } else { "no" });
    }
}

fn filter_devices<'a>(
    devices: &'a [Device],
    identifiers: &[String],
) -> Vec<&'a Device> {
    if identifiers.is_empty() {
        return devices.iter().collect();
    }

    let mut seen = vec![false; devices.len()];
    let mut result = Vec::new();
    for ident_str in identifiers {
        if let Some(ident) = parse_identifier(ident_str) {
            for (i, dev) in devices.iter().enumerate() {
                if device_matches(dev, &ident) && !seen[i] {
                    seen[i] = true;
                    result.push(dev);
                }
            }
        }
    }
    result
}

fn do_block_op(identifiers: &[String], soft: u8) -> ExitCode {
    for ident_str in identifiers {
        let ident = match parse_identifier(ident_str) {
            Some(i) => i,
            None => {
                eprintln!("rfkill: invalid identifier: {ident_str}");
                return ExitCode::FAILURE;
            }
        };

        let result = match ident {
            Identifier::Type(t) => write_event(0, t, OP_CHANGE_ALL, soft),
            Identifier::Id(id) => write_event(id, 0, OP_CHANGE, soft),
        };

        if let Err(e) = result {
            eprintln!("rfkill: {e}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn do_toggle(identifiers: &[String]) -> ExitCode {
    let devices = read_devices();

    for ident_str in identifiers {
        let ident = match parse_identifier(ident_str) {
            Some(i) => i,
            None => {
                eprintln!("rfkill: invalid identifier: {ident_str}");
                return ExitCode::FAILURE;
            }
        };

        for dev in &devices {
            if device_matches(dev, &ident) {
                let new_soft: u8 = if dev.soft { 0 } else { 1 };
                if let Err(e) = write_event(dev.id, 0, OP_CHANGE, new_soft) {
                    eprintln!("rfkill: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    ExitCode::SUCCESS
}

fn do_event() -> ExitCode {
    let mut f = match File::open("/dev/rfkill") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("rfkill: cannot open /dev/rfkill: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut buf = [0u8; 8];
    loop {
        match f.read_exact(&mut buf) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("rfkill: read error: {e}");
                return ExitCode::FAILURE;
            }
        }

        let idx = u32::from_ne_bytes(buf[0..4].try_into().unwrap());
        let rf_type = buf[4];
        let op = buf[5];
        let soft = buf[6];
        let hard = buf[7];

        let op_name = match op {
            OP_ADD => "add",
            OP_DEL => "remove",
            OP_CHANGE => "change",
            OP_CHANGE_ALL => "change-all",
            _ => "unknown",
        };

        println!(
            "{idx}: {}: {op_name} soft={} hard={}",
            type_to_name(rf_type),
            blocked_str(soft != 0),
            blocked_str(hard != 0),
        );
    }
    ExitCode::SUCCESS
}

pub fn run(args: Args) -> ExitCode {
    match &args.command {
        None => {
            let devices = read_devices();
            print_table_output(&args, &devices)
        }
        Some(Cmd::List { identifiers }) => {
            let devices = read_devices();
            if args.output.is_some() || args.output_all || args.json || args.raw
            {
                let filtered: Vec<&Device> =
                    filter_devices(&devices, identifiers);
                let owned: Vec<Device> = filtered
                    .into_iter()
                    .map(|d| Device {
                        id: d.id,
                        rf_type: d.rf_type,
                        name: d.name.clone(),
                        soft: d.soft,
                        hard: d.hard,
                    })
                    .collect();
                print_table_output(&args, &owned)
            } else {
                let filtered = filter_devices(&devices, identifiers);
                let refs: Vec<&Device> = filtered;
                let owned: Vec<Device> = refs
                    .into_iter()
                    .map(|d| Device {
                        id: d.id,
                        rf_type: d.rf_type,
                        name: d.name.clone(),
                        soft: d.soft,
                        hard: d.hard,
                    })
                    .collect();
                print_legacy_list(&owned);
                ExitCode::SUCCESS
            }
        }
        Some(Cmd::Block { identifiers }) => do_block_op(identifiers, 1),
        Some(Cmd::Unblock { identifiers }) => do_block_op(identifiers, 0),
        Some(Cmd::Toggle { identifiers }) => do_toggle(identifiers),
        Some(Cmd::Event) => do_event(),
    }
}
