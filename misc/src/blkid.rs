use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::{ArgAction, Parser};
use linuxutils_blkid::{Cache, Tag, TagKind};
use std::{collections::BTreeMap, path::PathBuf, process::ExitCode};

/// Locate or print block device attributes (LABEL, UUID, PARTLABEL,
/// PARTUUID, ID).
///
/// Resolution and listing go through the udev `/dev/disk/by-*`
/// symlink layer. The on-disk filesystem-signature probe path
/// (`-p, --probe`) is **not implemented** in this v0 — invocations
/// that need it will exit with a clear error rather than silently
/// returning incomplete data. Use upstream `blkid` for probing.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "blkid", about = "Locate/print block device attributes")]
pub struct Args {
    /// Look up the device that uses this filesystem label
    #[arg(short = 'L', long, value_name = "LABEL")]
    label: Option<String>,

    /// Look up the device with this UUID
    #[arg(short = 'U', long, value_name = "UUID")]
    uuid: Option<String>,

    /// Output format: `full` (default), `value`, `device`, `export`
    #[arg(short = 'o', long, default_value = "full", value_name = "FORMAT")]
    output: String,

    /// Only print the first match for -L / -U
    #[arg(short = 'l', long, action = ArgAction::SetTrue)]
    list_one: bool,

    /// Don't encode non-printing characters (no-op in v0; kept for
    /// command-line compatibility)
    #[arg(short = 'd', long, action = ArgAction::SetTrue)]
    no_encoding: bool,

    /// Cache file (ignored — v0 reads udev symlinks directly)
    #[arg(short = 'c', long, value_name = "FILE")]
    cache_file: Option<PathBuf>,

    /// Stub: probing isn't implemented in v0
    #[arg(short = 'p', long, action = ArgAction::SetTrue)]
    probe: bool,

    /// Stub: probing-info isn't implemented in v0
    #[arg(short = 'i', long, action = ArgAction::SetTrue)]
    info: bool,

    /// Devices to print attributes for; empty means list every
    /// device known to udev
    devices: Vec<PathBuf>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Format {
    Full,
    Value,
    Device,
    Export,
}

fn parse_format(s: &str) -> Result<Format, String> {
    match s {
        "full" => Ok(Format::Full),
        "value" => Ok(Format::Value),
        "device" => Ok(Format::Device),
        "export" => Ok(Format::Export),
        // upstream supports `udev`, `list`, `json` too; flag those
        // as not-yet-implemented so scripts get a clear failure
        // rather than misleading default-format output.
        "udev" | "list" | "json" => {
            Err(format!("output format {s:?} not implemented in this v0"))
        }
        _ => Err(format!("unknown output format: {s}")),
    }
}

fn print_one(device: &std::path::Path, tags: &[Tag], fmt: Format) {
    if tags.is_empty() {
        // Match upstream: silent when a device has no recognised
        // tags. Caller's exit status reflects whether anything was
        // printed.
        return;
    }
    match fmt {
        Format::Device => {
            println!("{}", device.display());
        }
        Format::Value => {
            for t in tags {
                println!("{}", t.value.to_string_lossy());
            }
        }
        Format::Export => {
            println!("DEVNAME={}", device.display());
            for t in tags {
                println!(
                    "{}={}",
                    t.kind.as_str(),
                    t.value.to_string_lossy()
                );
            }
            println!();
        }
        Format::Full => {
            print!("{}:", device.display());
            for t in tags {
                print!(
                    " {}=\"{}\"",
                    t.kind.as_str(),
                    t.value.to_string_lossy()
                );
            }
            println!();
        }
    }
}

fn lookup(cache: &Cache, kind: TagKind, value: &str) -> ExitCode {
    let tag = Tag::new(kind, value);
    match cache.resolve(&tag) {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        // Upstream returns exit 2 for "specified token was not found".
        Err(_) => ExitCode::from(2),
    }
}

pub fn run(args: Args) -> ExitCode {
    if args.probe || args.info {
        eprintln!(
            "blkid: --probe / --info require the on-disk signature \
             scanner, which is not yet implemented in this build"
        );
        return ExitCode::FAILURE;
    }
    let _ = args.no_encoding; // accepted, no-op
    let _ = args.cache_file; // accepted, ignored

    let cache = Cache::new();

    // Single-device shortcuts via -L / -U.
    if let Some(label) = &args.label {
        return lookup(&cache, TagKind::Label, label);
    }
    if let Some(uuid) = &args.uuid {
        return lookup(&cache, TagKind::Uuid, uuid);
    }

    let fmt = match parse_format(&args.output) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("blkid: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut printed_any = false;

    if args.devices.is_empty() {
        // Group every (Tag, device) pair by device so we print one
        // line per device with all its tags.
        let scan = cache.scan();
        let mut by_device: BTreeMap<PathBuf, Vec<Tag>> = BTreeMap::new();
        for (tag, path) in scan.entries {
            by_device.entry(path).or_default().push(tag);
        }
        for (device, mut tags) in by_device {
            // Stable order across kinds for predictable output.
            tags.sort_by_key(|t| t.kind as u8);
            if !tags.is_empty() {
                printed_any = true;
            }
            print_one(&device, &tags, fmt);
        }
    } else {
        for device in &args.devices {
            let mut tags = cache.tags_for(device);
            tags.sort_by_key(|t| t.kind as u8);
            if !tags.is_empty() {
                printed_any = true;
            }
            print_one(device, &tags, fmt);
        }
    }

    if printed_any {
        ExitCode::SUCCESS
    } else {
        // Upstream exit 2 when the device list is non-empty but
        // nothing matched.
        ExitCode::from(2)
    }
}
