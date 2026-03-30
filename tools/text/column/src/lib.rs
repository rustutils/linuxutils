use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process::ExitCode,
};

const TAB_WIDTH: usize = 8;

/// Columnate lists.
#[derive(Parser)]
#[command(name = "column", version, about)]
pub struct Args {
    /// Create a table from delimited input.
    #[arg(short = 't', long = "table")]
    table: bool,

    /// Possible input item delimiters (default is whitespace).
    #[arg(short = 's', long = "separator", value_name = "separators")]
    separator: Option<String>,

    /// Column delimiter for table output (default is two spaces).
    #[arg(short = 'o', long = "output-separator", value_name = "string")]
    output_separator: Option<String>,

    /// Specify column names (comma-separated).
    #[arg(short = 'N', long = "table-columns", value_name = "names")]
    table_columns: Option<String>,

    /// Omit printing the table header.
    #[arg(short = 'd', long = "table-noheadings")]
    table_noheadings: bool,

    /// Maximum number of input columns.
    #[arg(short = 'l', long = "table-columns-limit", value_name = "number")]
    table_columns_limit: Option<usize>,

    /// Fill all available space on output.
    #[arg(short = 'm', long = "table-maxout")]
    table_maxout: bool,

    /// Fill rows before columns.
    #[arg(short = 'x', long = "fillrows")]
    fill_rows: bool,

    /// Output width (default: terminal width or 80). Use 0 or "unlimited" for no limit.
    #[arg(short = 'c', long = "output-width", value_name = "width")]
    output_width: Option<String>,

    /// Use spaces instead of tabs, with this minimum spacing between columns.
    #[arg(short = 'S', long = "use-spaces", value_name = "number")]
    use_spaces: Option<usize>,

    /// Preserve blank lines in input.
    #[arg(short = 'L', long = "keep-empty-lines")]
    keep_empty_lines: bool,

    /// Files to read. If none are given, reads from standard input.
    files: Vec<PathBuf>,
}

fn terminal_width() -> usize {
    let stdout = io::stdout();
    match rustix::termios::tcgetwinsize(&stdout) {
        Ok(ws) if ws.ws_col > 0 => ws.ws_col as usize,
        _ => 80,
    }
}

fn parse_output_width(s: &str) -> Option<usize> {
    match s {
        "unlimited" | "0" => None,
        _ => s.parse::<usize>().ok(),
    }
}

fn round_up_to_tab(width: usize) -> usize {
    width.div_ceil(TAB_WIDTH) * TAB_WIDTH
}

fn pad_with_tabs(
    item: &str,
    col_width: usize,
    writer: &mut impl Write,
) -> io::Result<()> {
    writer.write_all(item.as_bytes())?;
    let chars_used = item.len();
    let target = round_up_to_tab(col_width);
    let mut pos = chars_used;
    while pos < target {
        let next_tab = ((pos / TAB_WIDTH) + 1) * TAB_WIDTH;
        writer.write_all(b"\t")?;
        pos = next_tab;
    }
    Ok(())
}

fn pad_with_spaces(
    item: &str,
    col_width: usize,
    spacing: usize,
    writer: &mut impl Write,
) -> io::Result<()> {
    write!(writer, "{:<width$}", item, width = col_width + spacing)?;
    Ok(())
}

pub fn run(args: Args) -> ExitCode {
    let width_limit = match &args.output_width {
        Some(s) => parse_output_width(s),
        None => Some(terminal_width()),
    };

    let input = match read_input(&args.files) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("column: {e}");
            return ExitCode::FAILURE;
        }
    };

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    let result = if args.table {
        table_mode(&input, &args, width_limit, &mut writer)
    } else {
        columnate(
            &input,
            width_limit,
            args.fill_rows,
            args.use_spaces,
            args.keep_empty_lines,
            &mut writer,
        )
    };

    if let Err(e) = result {
        eprintln!("column: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn read_input(files: &[PathBuf]) -> io::Result<String> {
    let mut buf = String::new();
    if files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();
        for line in reader.lines() {
            buf.push_str(&line?);
            buf.push('\n');
        }
    } else {
        for path in files {
            let file = File::open(path).map_err(|e| {
                io::Error::new(e.kind(), format!("{}: {e}", path.display()))
            })?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                buf.push_str(&line?);
                buf.push('\n');
            }
        }
    }
    Ok(buf)
}

fn table_mode(
    input: &str,
    args: &Args,
    width_limit: Option<usize>,
    writer: &mut impl Write,
) -> io::Result<()> {
    use cols::{Table, TermForce, print_table};

    // Parse input into rows of fields.
    let rows = parse_table_input(
        input,
        args.separator.as_deref(),
        args.table_columns_limit,
    );

    if rows.is_empty() {
        return Ok(());
    }

    // Determine column count from the widest row.
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return Ok(());
    }

    // Build column names.
    let col_names: Vec<String> = if let Some(ref names_str) = args.table_columns
    {
        let user_names: Vec<&str> = names_str.split(',').collect();
        (0..ncols)
            .map(|i| {
                user_names
                    .get(i)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("COL{}", i + 1))
            })
            .collect()
    } else {
        (0..ncols).map(|i| format!("COL{}", i + 1)).collect()
    };

    // Build the table.
    let mut table = Table::new();

    if let Some(width) = width_limit {
        table.set_termwidth(width);
        table.set_termforce(TermForce::Always);
    }

    if args.table_noheadings || args.table_columns.is_none() {
        table.enable_no_headings(true);
    }

    if args.table_maxout {
        table.enable_maxout(true);
    }

    if let Some(ref sep) = args.output_separator {
        table.set_column_separator(sep);
    }

    for name in &col_names {
        table.new_column(name);
    }

    // Populate rows.
    for row in &rows {
        let line_id = table.new_line(None);
        let line = table.line_mut(line_id);
        for (ci, cell) in row.iter().enumerate() {
            line.set_data(ci, cell);
        }
    }

    print_table(&table, writer)
}

/// Parse input text into rows of fields based on separator.
fn parse_table_input(
    input: &str,
    separator: Option<&str>,
    columns_limit: Option<usize>,
) -> Vec<Vec<String>> {
    let mut rows = Vec::new();

    for line in input.lines() {
        if line.is_empty() {
            continue;
        }

        let fields: Vec<String> = if let Some(sep) = separator {
            // Split by any character in the separator string (non-greedy).
            let sep_chars: Vec<char> = sep.chars().collect();
            split_by_chars(line, &sep_chars, columns_limit)
        } else {
            // Default: split by whitespace runs.
            split_by_whitespace(line, columns_limit)
        };

        rows.push(fields);
    }

    rows
}

/// Split a line by any character in `sep_chars`, keeping empty fields
/// (non-greedy splitting like util-linux >= 2.23).
fn split_by_chars(
    line: &str,
    sep_chars: &[char],
    limit: Option<usize>,
) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();

    for ch in line.chars() {
        if sep_chars.contains(&ch) {
            if let Some(max) = limit
                && fields.len() + 1 >= max
            {
                // Last field gets the rest.
                current.push(ch);
                continue;
            }
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    fields.push(current);
    fields
}

/// Split by whitespace runs (greedy), with optional column limit.
fn split_by_whitespace(line: &str, limit: Option<usize>) -> Vec<String> {
    match limit {
        Some(max) => {
            let mut fields: Vec<String> = Vec::new();
            let mut rest = line;
            while fields.len() + 1 < max {
                rest = rest.trim_start();
                if rest.is_empty() {
                    break;
                }
                if let Some(pos) = rest.find(char::is_whitespace) {
                    fields.push(rest[..pos].to_string());
                    rest = &rest[pos..];
                } else {
                    fields.push(rest.to_string());
                    rest = "";
                    break;
                }
            }
            if !rest.is_empty() {
                fields.push(rest.trim_start().to_string());
            }
            fields
        }
        None => line.split_whitespace().map(String::from).collect(),
    }
}

pub fn columnate(
    input: &str,
    width_limit: Option<usize>,
    fill_rows: bool,
    use_spaces: Option<usize>,
    keep_empty_lines: bool,
    writer: &mut impl Write,
) -> io::Result<()> {
    let mut items: Vec<&str> = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            if keep_empty_lines {
                items.push("");
            }
            continue;
        }
        for word in line.split_whitespace() {
            items.push(word);
        }
    }

    if items.is_empty() {
        return Ok(());
    }

    let max_item_width = items.iter().map(|s| s.len()).max().unwrap_or(0);

    let (num_cols, col_width) = match width_limit {
        None => (items.len(), max_item_width),
        Some(width) => {
            let effective_col_width = match use_spaces {
                Some(spacing) => max_item_width + spacing,
                None => round_up_to_tab(max_item_width + 1).max(TAB_WIDTH),
            };
            if effective_col_width == 0 || effective_col_width > width {
                (1, max_item_width)
            } else {
                let cols = width / effective_col_width;
                (cols.max(1), max_item_width)
            }
        }
    };

    let num_rows = items.len().div_ceil(num_cols);

    for row in 0..num_rows {
        let mut first = true;
        for col in 0..num_cols {
            let idx = if fill_rows {
                row * num_cols + col
            } else {
                col * num_rows + row
            };

            if idx >= items.len() {
                continue;
            }

            let item = items[idx];

            if item.is_empty() && keep_empty_lines {
                writeln!(writer)?;
                first = true;
                continue;
            }

            if !first {
                // Padding was already applied to the previous item
            } else {
                first = false;
            }

            let is_last_in_row = {
                let mut last = true;
                for next_col in (col + 1)..num_cols {
                    let next_idx = if fill_rows {
                        row * num_cols + next_col
                    } else {
                        next_col * num_rows + row
                    };
                    if next_idx < items.len() {
                        last = false;
                        break;
                    }
                }
                last
            };

            if is_last_in_row {
                writer.write_all(item.as_bytes())?;
            } else {
                match use_spaces {
                    Some(spacing) => {
                        pad_with_spaces(item, col_width, spacing, writer)?
                    }
                    None => pad_with_tabs(item, col_width, writer)?,
                }
            }
        }
        writeln!(writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_columnate(
        input: &str,
        width: Option<usize>,
        fill_rows: bool,
        use_spaces: Option<usize>,
        keep_empty_lines: bool,
    ) -> String {
        let mut output = Vec::new();
        columnate(
            input,
            width,
            fill_rows,
            use_spaces,
            keep_empty_lines,
            &mut output,
        )
        .unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn fill_columns_default() {
        let input = "a\nb\nc\nd\ne\nf\n";
        let result = run_columnate(input, Some(10), false, Some(2), false);
        assert_eq!(result, "a  c  e\nb  d  f\n");
    }

    #[test]
    fn fill_rows() {
        let input = "a\nb\nc\nd\ne\nf\n";
        let result = run_columnate(input, Some(10), true, Some(2), false);
        assert_eq!(result, "a  b  c\nd  e  f\n");
    }

    #[test]
    fn empty_input() {
        let result = run_columnate("", Some(80), false, Some(2), false);
        assert_eq!(result, "");
    }

    #[test]
    fn single_item() {
        let result = run_columnate("hello\n", Some(80), false, Some(2), false);
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn no_width_limit() {
        let input = "a\nb\nc\nd\n";
        let result = run_columnate(input, None, false, Some(2), false);
        assert_eq!(result, "a  b  c  d\n");
    }

    #[test]
    fn keep_empty_lines() {
        let input = "a\n\nb\nc\n";
        let result = run_columnate(input, Some(80), false, Some(2), true);
        // Empty line becomes an empty item, printed on its own line
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains("c"));
    }

    #[test]
    fn uneven_items() {
        let input = "a\nb\nc\nd\ne\n";
        let result = run_columnate(input, Some(10), false, Some(2), false);
        // 5 items, 3 cols => 2 rows
        // col-first: row0=[a,c,e] row1=[b,d]
        assert_eq!(result, "a  c  e\nb  d\n");
    }
}
