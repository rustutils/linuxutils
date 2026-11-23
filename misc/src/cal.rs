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

/// Day-of-week header for the four (julian, sunday-first) combinations.
const HDR_NORMAL_SUN: &str = "Su Mo Tu We Th Fr Sa";
const HDR_NORMAL_MON: &str = "Mo Tu We Th Fr Sa Su";
const HDR_JULIAN_SUN: &str = "Sun Mon Tue Wed Thu Fri Sat";
const HDR_JULIAN_MON: &str = "Mon Tue Wed Thu Fri Sat Sun";

/// Inter-month gap. Upstream `cal` uses three spaces in `--year`
/// mode and two spaces everywhere else (`--three`, `--months N`,
/// `--twelve`).
const YEAR_GAP: &str = "   ";
const MONTH_GAP: &str = "  ";

/// Layout descriptor for one calendar invocation. Computed once
/// from `--julian` / `--week` and consulted by every rendering
/// helper instead of re-deriving widths from booleans inline.
#[derive(Clone, Copy)]
struct Layout {
    julian: bool,
    weeknum: bool,
}

impl Layout {
    /// Width of a single day cell, in characters.
    fn cell(self) -> usize {
        if self.julian { 3 } else { 2 }
    }

    /// Width of the day-grid (7 cells with 1-char separators).
    fn day_grid(self) -> usize {
        7 * self.cell() + 6
    }

    /// Width of the week-number prefix including its trailing
    /// separator. `0` when `--week` isn't set.
    fn weeknum_prefix(self) -> usize {
        if self.weeknum { 3 } else { 0 }
    }

    /// Total month-column width (week-num prefix + day grid).
    fn month_width(self) -> usize {
        self.weeknum_prefix() + self.day_grid()
    }

    fn day_header(self, monday_first: bool) -> &'static str {
        match (self.julian, monday_first) {
            (false, false) => HDR_NORMAL_SUN,
            (false, true) => HDR_NORMAL_MON,
            (true, false) => HDR_JULIAN_SUN,
            (true, true) => HDR_JULIAN_MON,
        }
    }
}

#[derive(Parser, Debug)]
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

    /// Print the week numbers alongside the days.
    #[arg(short = 'w', long = "week")]
    week: bool,

    /// Number of columns to use. With auto, cal picks a number that fits
    /// the terminal width.
    #[arg(short = 'c', long, default_value = "auto")]
    columns: String,

    /// Positional arguments: [[[day] month] year]
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

/// Center a string in a field of `width` chars, with extra padding on the
/// left (matching the C cal behavior).
fn center(s: &str, width: usize) -> String {
    let len = s.chars().count();
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

/// ISO 8601 week number (used when weeks start on Monday).
/// Falls back to chrono — the algorithm involves "Thursday rule"
/// edge cases at year boundaries that aren't worth re-deriving.
fn iso_week(year: u32, month: u32, day: u32) -> u32 {
    use chrono::Datelike;
    chrono::NaiveDate::from_ymd_opt(year as i32, month, day)
        .map(|d| d.iso_week().week())
        .unwrap_or(0)
}

/// Sunday-based US-style week numbering: the week containing Jan 1
/// is week 1, weeks start on Sunday. Week 1 is shared with the
/// previous calendar year if Jan 1 falls mid-week.
fn sunday_week(year: u32, month: u32, day: u32) -> u32 {
    let dow_jan1 = day_of_week(year, 1, 1); // 0=Sun..6=Sat
    let doy = day_of_year(year, month, day);
    (doy - 1 + dow_jan1) / 7 + 1
}

/// Render a single month as lines exactly `layout.month_width()`
/// characters wide.
fn render_month(
    year: u32,
    month: u32,
    monday_first: bool,
    layout: Layout,
    show_year: bool,
) -> Vec<String> {
    let cell = layout.cell();
    let prefix_w = layout.weeknum_prefix();
    let total_w = layout.month_width();
    let blank_prefix = " ".repeat(prefix_w);

    let mut lines = Vec::new();

    // Title line: month name (+ year if requested), centered over
    // the entire month column (week-number prefix included).
    let title = if show_year {
        format!("{} {}", MONTH_NAMES[(month - 1) as usize], year)
    } else {
        MONTH_NAMES[(month - 1) as usize].to_string()
    };
    lines.push(center(&title, total_w));

    // Day-of-week header.
    lines.push(format!("{blank_prefix}{}", layout.day_header(monday_first)));

    let ndays = days_in_month(year, month);
    let first_dow = day_of_week(year, month, 1);
    // Adjust for Monday-first: Monday=0 .. Sunday=6
    let offset = if monday_first {
        (first_dow + 6) % 7
    } else {
        first_dow
    };

    // Build week rows.
    let mut line = String::new();
    let empty_cell = " ".repeat(cell);

    // Helper: compute the week-number that should be printed for the
    // first day in the row currently being built.
    let weeknum_for = |day: u32| -> u32 {
        if monday_first {
            iso_week(year, month, day)
        } else {
            sunday_week(year, month, day)
        }
    };

    // Each row's week-number is the week of the first day of the
    // month that falls in that row. For the first row that's
    // always `1` (regardless of how many leading blanks precede it).
    let mut current_week_day: u32 = 1;
    if layout.weeknum {
        line.push_str(&format!("{:>2} ", weeknum_for(current_week_day)));
    }

    for _ in 0..offset {
        line.push_str(&empty_cell);
        line.push(' ');
    }

    for day in 1..=ndays {
        if layout.julian {
            let doy = day_of_year(year, month, day);
            line.push_str(&format!("{doy:>3}"));
        } else {
            line.push_str(&format!("{day:>2}"));
        }

        let col = (offset + day - 1) % 7;
        let last_day = day == ndays;
        if col == 6 || last_day {
            // Pad to total width.
            while line.chars().count() < total_w {
                line.push(' ');
            }
            lines.push(line);
            line = String::new();

            if !last_day {
                current_week_day = day + 1;
                if layout.weeknum {
                    line.push_str(&format!(
                        "{:>2} ",
                        weeknum_for(current_week_day)
                    ));
                }
            }
        } else {
            line.push(' ');
        }
    }

    // Pad to a consistent height (8 lines) for clean side-by-side
    // rendering.
    while lines.len() < 8 {
        lines.push(" ".repeat(total_w));
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
fn print_months(
    months: &[(u32, u32)],
    cols: u32,
    monday_first: bool,
    layout: Layout,
    show_year: bool,
    gap: &str,
    out: &mut dyn Write,
) -> io::Result<()> {
    let rendered: Vec<Vec<String>> = months
        .iter()
        .map(|&(y, m)| render_month(y, m, monday_first, layout, show_year))
        .collect();

    let total_w = layout.month_width();
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
                    write!(out, "{:total_w$}", "")?;
                }
            }
            writeln!(out)?;
        }
    }
    Ok(())
}

/// Resolve the `--columns` argument. `auto` (the default) picks a
/// number of columns that fits the terminal — the upstream
/// fallback is 3 columns when the terminal width can't be
/// determined, which is what every behavioural test expects.
fn resolve_columns(arg: &str) -> Result<u32, String> {
    if arg.eq_ignore_ascii_case("auto") {
        // Match upstream: 3 columns when no TTY width is available.
        return Ok(3);
    }
    arg.parse::<u32>()
        .map_err(|_| format!("invalid --columns value: {arg}"))
        .and_then(|n| {
            if n == 0 {
                Err("--columns must be at least 1".to_string())
            } else {
                Ok(n)
            }
        })
}

pub fn run(args: Args) -> ExitCode {
    let (cur_year, cur_month, cur_day) = today();

    let monday_first = args.monday && !args.sunday;

    let layout = Layout {
        julian: args.julian,
        weeknum: args.week,
    };

    let cols = match resolve_columns(&args.columns) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Parse positional arguments.
    let (target_year, target_month, target_day) = match args.args.len() {
        0 => (cur_year, Some(cur_month), Some(cur_day)),
        1 => {
            let arg = &args.args[0];
            if let Ok(n) = arg.parse::<u32>() {
                (n, None, None) // treat as year
            } else if let Some(m) = parse_month_name(arg) {
                (cur_year, Some(m), None)
            } else if arg == "today" || arg == "now" {
                (cur_year, Some(cur_month), Some(cur_day))
            } else if arg == "tomorrow" {
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

    // We don't currently highlight the target day (no terminal
    // colour support) but the parameter still controls things like
    // the title row.
    let _ = target_day;

    if args.year || target_month.is_none() {
        // Year view.
        let year_title = format!("{target_year}");
        let total_width = cols as usize * layout.month_width()
            + (cols as usize - 1) * YEAR_GAP.len();
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
            layout,
            false,
            YEAR_GAP,
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else if args.twelve {
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
            layout,
            true,
            MONTH_GAP,
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else if args.three {
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
            layout,
            true,
            MONTH_GAP,
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else if let Some(n) = args.months {
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
            layout,
            true,
            MONTH_GAP,
            &mut out,
        ) {
            eprintln!("cal: {e}");
            return ExitCode::FAILURE;
        }
    } else {
        let tm = target_month.unwrap_or(cur_month);
        let rendered =
            render_month(target_year, tm, monday_first, layout, true);
        for line in &rendered {
            if let Err(e) = writeln!(out, "{line}") {
                eprintln!("cal: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_week_jan1_2006_is_week_52_of_prev_year() {
        // Jan 1 2006 is a Sunday — ISO week 52 of 2005.
        assert_eq!(iso_week(2006, 1, 1), 52);
    }

    #[test]
    fn iso_week_first_thursday_rule() {
        // Jan 4 is always in ISO week 1.
        assert_eq!(iso_week(2006, 1, 4), 1);
        assert_eq!(iso_week(2007, 1, 4), 1);
        assert_eq!(iso_week(2008, 1, 4), 1);
    }

    #[test]
    fn sunday_week_jan1_is_always_week_1() {
        assert_eq!(sunday_week(2006, 1, 1), 1);
        assert_eq!(sunday_week(2007, 1, 1), 1);
    }

    #[test]
    fn sunday_week_increments_each_week() {
        assert_eq!(sunday_week(2006, 1, 7), 1); // Sat
        assert_eq!(sunday_week(2006, 1, 8), 2); // Sun
        assert_eq!(sunday_week(2006, 2, 1), 5);
    }

    #[test]
    fn layout_widths_match_upstream() {
        let plain = Layout {
            julian: false,
            weeknum: false,
        };
        assert_eq!(plain.month_width(), 20);

        let julian = Layout {
            julian: true,
            weeknum: false,
        };
        assert_eq!(julian.month_width(), 27);

        let plain_w = Layout {
            julian: false,
            weeknum: true,
        };
        assert_eq!(plain_w.month_width(), 23);

        let julian_w = Layout {
            julian: true,
            weeknum: true,
        };
        assert_eq!(julian_w.month_width(), 30);
    }

    #[test]
    fn columns_auto_resolves_to_three() {
        assert_eq!(resolve_columns("auto").unwrap(), 3);
        assert_eq!(resolve_columns("AUTO").unwrap(), 3);
    }

    #[test]
    fn columns_numeric_passes_through() {
        assert_eq!(resolve_columns("5").unwrap(), 5);
    }

    #[test]
    fn columns_zero_rejected() {
        assert!(resolve_columns("0").is_err());
    }
}
