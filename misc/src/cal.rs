use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{self, Write},
    process::ExitCode,
};

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const MONTH_ABBREVS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct",
    "nov", "dec",
];

const DAYS_HEADER_SUN: &str = "Su Mo Tu We Th Fr Sa";
const DAYS_HEADER_MON: &str = "Mo Tu We Th Fr Sa Su";
const MONTH_WIDTH: usize = 20;
const MONTH_GAP: &str = "  ";

#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "cal", about = "Display a calendar")]
pub struct Args {
    /// Display single month output (default)
    #[arg(short = '1', long)]
    one: bool,

    /// Display three months spanning the date
    #[arg(short = '3', long)]
    three: bool,

    /// Display number of months starting from the date
    #[arg(short = 'n', long)]
    months: Option<u32>,

    /// Display Sunday as the first day of the week
    #[arg(short, long)]
    sunday: bool,

    /// Display Monday as the first day of the week
    #[arg(short, long)]
    monday: bool,

    /// Display a calendar for the whole year
    #[arg(short = 'y', long)]
    year: bool,

    /// Display a calendar for the next twelve months
    #[arg(short = 'Y', long)]
    twelve: bool,

    /// Use day-of-year (ordinal) numbering
    #[arg(short, long)]
    julian: bool,

    /// Number of columns to use
    #[arg(short = 'c', long, default_value = "3")]
    columns: u32,

    /// Positional arguments: [[[day] month] year]
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

/// Center a string in a field of `width` chars, with extra padding on the
/// left (matching the C cal behavior).
fn center(s: &str, width: usize) -> String {
    let len = s.len();
    if len >= width {
        return s.to_string();
    }
    let total_pad = width - len;
    let left = total_pad.div_ceil(2);
    let right = total_pad / 2;
    format!("{:left$}{s}{:right$}", "", "")
}

fn today() -> (u32, u32, u32) {
    let now = chrono::Local::now().date_naive();
    use chrono::Datelike;
    (now.year() as u32, now.month(), now.day())
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100))
        || year.is_multiple_of(400)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Day of week for a given date (0=Sunday, 6=Saturday). Uses Zeller's formula
/// for the Gregorian calendar.
fn day_of_week(year: u32, month: u32, day: u32) -> u32 {
    let (y, m) = if month <= 2 {
        (year as i32 - 1, month as i32 + 12)
    } else {
        (year as i32, month as i32)
    };
    let q = day as i32;
    let k = y % 100;
    let j = y / 100;
    let h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // h: 0=Saturday, 1=Sunday, ..., 6=Friday → convert to 0=Sunday
    ((h + 6) % 7) as u32
}

/// Day of year (1-based).
fn day_of_year(year: u32, month: u32, day: u32) -> u32 {
    let mut doy = 0;
    for m in 1..month {
        doy += days_in_month(year, m);
    }
    doy + day
}

/// Render a single month as lines of exactly MONTH_WIDTH characters.
/// If `show_year` is true, the title includes the year.
fn render_month(
    year: u32,
    month: u32,
    monday_first: bool,
    julian: bool,
    highlight_day: Option<u32>,
    show_year: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    // Title line: centered month name (+ year if requested).
    let title = if show_year {
        format!("{} {}", MONTH_NAMES[(month - 1) as usize], year)
    } else {
        MONTH_NAMES[(month - 1) as usize].to_string()
    };
    lines.push(center(&title, MONTH_WIDTH));

    // Day-of-week header.
    if monday_first {
        lines.push(DAYS_HEADER_MON.to_string());
    } else {
        lines.push(DAYS_HEADER_SUN.to_string());
    }

    let ndays = days_in_month(year, month);
    let first_dow = day_of_week(year, month, 1);
    // Adjust for Monday-first: Monday=0 .. Sunday=6
    let offset = if monday_first {
        (first_dow + 6) % 7
    } else {
        first_dow
    };

    let mut line = String::new();
    // Leading blanks.
    for _ in 0..offset {
        line.push_str("   ");
    }

    let _ = highlight_day; // TODO: terminal highlighting

    for day in 1..=ndays {
        if julian {
            let doy = day_of_year(year, month, day);
            line.push_str(&format!("{doy:>3}"));
        } else {
            line.push_str(&format!("{day:>2}"));
        }

        let col = (offset + day - 1) % 7;
        if col == 6 || day == ndays {
            // Pad to MONTH_WIDTH.
            while line.len() < MONTH_WIDTH {
                line.push(' ');
            }
            lines.push(line);
            line = String::new();
        } else {
            line.push(' ');
        }
    }

    // Ensure we always have 8 lines (title + header + 6 week rows) for
    // consistent multi-month layout.
    while lines.len() < 8 {
        lines.push(" ".repeat(MONTH_WIDTH));
    }

    lines
}

fn parse_month_name(s: &str) -> Option<u32> {
    let lower = s.to_lowercase();
    for (i, abbrev) in MONTH_ABBREVS.iter().enumerate() {
        if lower.starts_with(abbrev) {
            return Some((i + 1) as u32);
        }
    }
    None
}

/// Print multiple months side by side, `cols` columns wide.
#[allow(clippy::too_many_arguments)]
fn print_months(
    months: &[(u32, u32)], // (year, month) pairs
    cols: u32,
    monday_first: bool,
    julian: bool,
    highlight: Option<(u32, u32, u32)>, // (year, month, day) to highlight
    show_year: bool,
    gap: &str,
    out: &mut dyn Write,
) -> io::Result<()> {
    let rendered: Vec<Vec<String>> = months
        .iter()
        .map(|&(y, m)| {
            let hl = highlight.and_then(|(hy, hm, hd)| {
                if hy == y && hm == m { Some(hd) } else { None }
            });
            render_month(y, m, monday_first, julian, hl, show_year)
        })
        .collect();

    for chunk in rendered.chunks(cols as usize) {
        let max_lines = chunk.iter().map(|m| m.len()).max().unwrap_or(0);
        for row in 0..max_lines {
            for (ci, month) in chunk.iter().enumerate() {
                if ci > 0 {
                    write!(out, "{gap}")?;
                }
                if row < month.len() {
                    write!(out, "{}", month[row])?;
                } else {
                    write!(out, "{:MONTH_WIDTH$}", "")?;
                }
            }
            writeln!(out)?;
        }
    }
    Ok(())
}

pub fn run(args: Args) -> ExitCode {
    let (cur_year, cur_month, cur_day) = today();

    let monday_first = args.monday && !args.sunday;

    // Parse positional arguments.
    let (target_year, target_month, target_day) = match args.args.len() {
        0 => (cur_year, Some(cur_month), Some(cur_day)),
        1 => {
            let arg = &args.args[0];
            if let Ok(n) = arg.parse::<u32>() {
                if (1..=12).contains(&n) {
                    // Ambiguous: could be month or year. `cal` treats a
                    // single small number as a year (cal 12 shows year 12).
                    // But month names are also accepted.
                    (n, None, None) // treat as year
                } else {
                    (n, None, None)
                }
            } else if let Some(m) = parse_month_name(arg) {
                (cur_year, Some(m), None)
            } else if arg == "today" || arg == "now" {
                (cur_year, Some(cur_month), Some(cur_day))
            } else if arg == "tomorrow" {
                // Simple: just advance one day.
                let mut d = cur_day + 1;
                let mut m = cur_month;
                let mut y = cur_year;
                if d > days_in_month(y, m) {
                    d = 1;
                    m += 1;
                    if m > 12 {
                        m = 1;
                        y += 1;
                    }
                }
                (y, Some(m), Some(d))
            } else if arg == "yesterday" {
                let mut d = cur_day as i32 - 1;
                let mut m = cur_month;
                let mut y = cur_year;
                if d < 1 {
                    m -= 1;
                    if m < 1 {
                        m = 12;
                        y -= 1;
                    }
                    d = days_in_month(y, m) as i32;
                }
                (y, Some(m), Some(d as u32))
            } else {
                eprintln!("cal: invalid argument: {arg}");
                return ExitCode::FAILURE;
            }
        }
        2 => {
            let month = match args.args[0].parse::<u32>() {
                Ok(m) if (1..=12).contains(&m) => m,
                _ => match parse_month_name(&args.args[0]) {
                    Some(m) => m,
                    None => {
                        eprintln!("cal: invalid month: {}", args.args[0]);
                        return ExitCode::FAILURE;
                    }
                },
            };
            let year = match args.args[1].parse::<u32>() {
                Ok(y) => y,
                Err(_) => {
                    eprintln!("cal: invalid year: {}", args.args[1]);
                    return ExitCode::FAILURE;
                }
            };
            (year, Some(month), None)
        }
        3 => {
            let day = match args.args[0].parse::<u32>() {
                Ok(d) => d,
                Err(_) => {
                    eprintln!("cal: invalid day: {}", args.args[0]);
                    return ExitCode::FAILURE;
                }
            };
            let month = match args.args[1].parse::<u32>() {
                Ok(m) if (1..=12).contains(&m) => m,
                _ => match parse_month_name(&args.args[1]) {
                    Some(m) => m,
                    None => {
                        eprintln!("cal: invalid month: {}", args.args[1]);
                        return ExitCode::FAILURE;
                    }
                },
            };
            let year = match args.args[2].parse::<u32>() {
                Ok(y) => y,
                Err(_) => {
                    eprintln!("cal: invalid year: {}", args.args[2]);
                    return ExitCode::FAILURE;
                }
            };
            (year, Some(month), Some(day))
        }
        _ => {
            eprintln!("cal: too many arguments");
            return ExitCode::FAILURE;
        }
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let highlight =
        target_day.map(|d| (target_year, target_month.unwrap_or(cur_month), d));

    let cols = args.columns;

    if args.year || target_month.is_none() {
        // Year view.
        let year_title = format!("{target_year}");
        let year_gap = "   ";
        let total_width =
            cols as usize * MONTH_WIDTH + (cols as usize - 1) * year_gap.len();
        if let Err(e) = writeln!(out, "{}", center(&year_title, total_width)) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
        if let Err(e) = writeln!(out) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }

        let months: Vec<(u32, u32)> =
            (1..=12).map(|m| (target_year, m)).collect();
        if let Err(e) = print_months(
            &months,
            cols,
            monday_first,
            args.julian,
            highlight,
            false,
            year_gap,
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else if args.twelve {
        // Next 12 months.
        let mut months = Vec::new();
        let mut y = target_year;
        let mut m = target_month.unwrap_or(cur_month);
        for _ in 0..12 {
            months.push((y, m));
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
        if let Err(e) = print_months(
            &months,
            cols,
            monday_first,
            args.julian,
            highlight,
            true,
            "  ",
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else if args.three {
        // Three months centered on the target.
        let tm = target_month.unwrap_or(cur_month);
        let mut months = Vec::new();
        for offset in [-1i32, 0, 1] {
            let mut m = tm as i32 + offset;
            let mut y = target_year as i32;
            if m < 1 {
                m += 12;
                y -= 1;
            } else if m > 12 {
                m -= 12;
                y += 1;
            }
            months.push((y as u32, m as u32));
        }
        if let Err(e) = print_months(
            &months,
            3,
            monday_first,
            args.julian,
            highlight,
            true,
            MONTH_GAP,
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else if let Some(n) = args.months {
        // N months starting from the target.
        let tm = target_month.unwrap_or(cur_month);
        let mut months = Vec::new();
        let mut y = target_year;
        let mut m = tm;
        for _ in 0..n {
            months.push((y, m));
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
        if let Err(e) = print_months(
            &months,
            cols,
            monday_first,
            args.julian,
            highlight,
            true,
            "  ",
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else {
        // Single month.
        let tm = target_month.unwrap_or(cur_month);
        let rendered = render_month(
            target_year,
            tm,
            monday_first,
            args.julian,
            target_day,
            true,
        );
        for line in &rendered {
            if let Err(e) = writeln!(out, "{line}") {
                eprintln!("cal: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
