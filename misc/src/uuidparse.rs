use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{self, BufRead},
    process::ExitCode,
};
use uuid::Uuid;

const ALL_COLUMNS: &[&str] = &["UUID", "VARIANT", "TYPE", "TIME"];

#[derive(Parser)]
#[command(
    name = "uuidparse",
    version,
    about = "A utility to parse unique identifiers",
    after_help = "Available output columns:\n     UUID  unique identifier\n  VARIANT  variant name\n     TYPE  type name\n     TIME  timestamp"
)]
pub struct Args {
    /// Use JSON output format
    #[arg(short = 'J', long = "json")]
    json: bool,

    /// Do not print a header line
    #[arg(short = 'n', long = "noheadings")]
    noheadings: bool,

    /// Specify which output columns to print (UUID,VARIANT,TYPE,TIME)
    #[arg(short = 'o', long = "output", value_delimiter = ',')]
    output: Option<Vec<String>>,

    /// Use the raw output format
    #[arg(short = 'r', long = "raw")]
    raw: bool,

    /// UUIDs to parse
    pub uuids: Vec<String>,
}

struct UuidInfo {
    uuid: String,
    variant: String,
    type_name: String,
    time: String,
}

pub fn run(args: Args) -> ExitCode {
    let columns = match &args.output {
        Some(cols) => {
            let mut result = Vec::new();
            for col in cols {
                let upper = col.to_uppercase();
                if ALL_COLUMNS.contains(&upper.as_str()) {
                    result.push(upper);
                } else {
                    eprintln!("uuidparse: unknown column: {col}");
                    return ExitCode::FAILURE;
                }
            }
            result
        }
        None => ALL_COLUMNS.iter().map(|s| s.to_string()).collect(),
    };

    let mut inputs: Vec<String> = args.uuids.clone();

    // If no arguments, read from stdin.
    if inputs.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            for word in line.split_whitespace() {
                inputs.push(word.to_string());
            }
        }
    }

    if inputs.is_empty() {
        return ExitCode::SUCCESS;
    }

    let infos: Vec<UuidInfo> =
        inputs.iter().map(|s| parse_uuid_info(s)).collect();

    if args.json {
        print_json(&infos, &columns);
    } else if args.raw {
        print_table(&infos, &columns, !args.noheadings, true);
    } else {
        print_table(&infos, &columns, !args.noheadings, false);
    }

    ExitCode::SUCCESS
}

fn parse_uuid_info(s: &str) -> UuidInfo {
    let (variant, type_name, time) = match Uuid::parse_str(s) {
        Ok(u) => {
            let variant = variant_name(&u);
            let type_name = type_name(&u);
            let time = format_time(&u);
            (variant, type_name, time)
        }
        Err(_) => ("invalid".to_string(), "invalid".to_string(), String::new()),
    };

    UuidInfo {
        uuid: s.to_string(),
        variant,
        type_name,
        time,
    }
}

fn variant_name(u: &Uuid) -> String {
    match u.get_variant() {
        uuid::Variant::NCS => "NCS".to_string(),
        uuid::Variant::RFC4122 => "DCE".to_string(),
        uuid::Variant::Microsoft => "Microsoft".to_string(),
        uuid::Variant::Future => "other".to_string(),
        _ => "other".to_string(),
    }
}

fn type_name(u: &Uuid) -> String {
    if u.is_nil() {
        return "nil".to_string();
    }
    match u.get_version() {
        Some(uuid::Version::Mac) => "time-based".to_string(),
        Some(uuid::Version::Dce) => "DCE".to_string(),
        Some(uuid::Version::Md5) => "name-based".to_string(),
        Some(uuid::Version::Random) => "random".to_string(),
        Some(uuid::Version::Sha1) => "sha1-based".to_string(),
        Some(uuid::Version::SortMac) => "time-v6".to_string(),
        Some(uuid::Version::SortRand) => "time-v7".to_string(),
        _ => "unknown".to_string(),
    }
}

fn format_time(u: &Uuid) -> String {
    let ts = match u.get_timestamp() {
        Some(ts) => ts,
        None => return String::new(),
    };

    let (secs, nanos) = ts.to_unix();
    // Format as ISO-ish timestamp matching util-linux output.
    let total_secs = secs as i64;
    let micros = nanos / 1000;

    // Use libc to get local time.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&total_secs as *const i64, &mut tm) };

    let mut tz_buf = [0u8; 8];
    let tz_len = unsafe {
        libc::strftime(
            tz_buf.as_mut_ptr() as *mut libc::c_char,
            tz_buf.len(),
            c"%z".as_ptr(),
            &tm,
        )
    };
    let tz = std::str::from_utf8(&tz_buf[..tz_len]).unwrap_or("");
    // Insert colon: +0200 -> +02:00
    let tz_formatted = if tz.len() == 5 {
        format!("{}:{}", &tz[..3], &tz[3..])
    } else {
        tz.to_string()
    };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02},{:06}{}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec,
        micros,
        tz_formatted,
    )
}

fn column_value<'a>(info: &'a UuidInfo, col: &str) -> &'a str {
    match col {
        "UUID" => &info.uuid,
        "VARIANT" => &info.variant,
        "TYPE" => &info.type_name,
        "TIME" => &info.time,
        _ => "",
    }
}

fn min_column_width(col: &str) -> usize {
    // Match util-linux's column widths (libsmartcols).
    // These include trailing padding to match the original output.
    match col {
        "UUID" => 37,
        "VARIANT" => 7,
        "TYPE" => 10,
        "TIME" => 4,
        _ => col.len(),
    }
}

fn print_table(
    infos: &[UuidInfo],
    columns: &[String],
    header: bool,
    raw: bool,
) {
    // Calculate column widths.
    let mut widths: Vec<usize> =
        columns.iter().map(|c| min_column_width(c)).collect();
    if !raw {
        for info in infos {
            for (i, col) in columns.iter().enumerate() {
                widths[i] = widths[i].max(column_value(info, col).len());
            }
        }
    }

    let sep = " ";

    if header {
        let parts: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                if raw || i == columns.len() - 1 {
                    col.to_string()
                } else {
                    format!("{:<width$}", col, width = widths[i])
                }
            })
            .collect();
        println!("{}", parts.join(sep));
    }

    for info in infos {
        let parts: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let val = column_value(info, col);
                if raw || i == columns.len() - 1 {
                    val.to_string()
                } else {
                    format!("{:<width$}", val, width = widths[i])
                }
            })
            .collect();
        println!("{}", parts.join(sep));
    }
}

fn print_json(infos: &[UuidInfo], columns: &[String]) {
    println!("{{");
    println!("   \"uuids\": [");
    for (i, info) in infos.iter().enumerate() {
        println!("      {{");
        let mut first = true;
        for col in columns {
            if !first {
                println!(",");
            }
            first = false;
            let key = col.to_lowercase();
            let val = column_value(info, col);
            if val.is_empty() {
                print!("         \"{key}\": null");
            } else {
                print!("         \"{key}\": \"{val}\"");
            }
        }
        println!();
        if i + 1 < infos.len() {
            println!("      }},");
        } else {
            println!("      }}");
        }
    }
    println!("   ]");
    println!("}}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_v4() {
        let info = parse_uuid_info("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(info.variant, "DCE");
        assert_eq!(info.type_name, "random");
    }

    #[test]
    fn parse_nil() {
        let info = parse_uuid_info("00000000-0000-0000-0000-000000000000");
        assert_eq!(info.type_name, "nil");
    }

    #[test]
    fn parse_random() {
        let u = Uuid::new_v4();
        let info = parse_uuid_info(&u.to_string());
        assert_eq!(info.variant, "DCE");
        assert_eq!(info.type_name, "random");
    }

    #[test]
    fn parse_invalid() {
        let info = parse_uuid_info("not-a-uuid");
        assert_eq!(info.variant, "invalid");
        assert_eq!(info.type_name, "invalid");
    }
}
