use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process::ExitCode,
};

const TAB_WIDTH: usize = 8;

/// Columnate lists.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "column", version)]
#[command(allow_negative_numbers = true)]
pub struct Args {
    /// Create a table from delimited input.
    #[arg(short = 't', long = "table")]
    table: bool,

    /// Use JSON output format (requires -N).
    #[arg(short = 'J', long = "json")]
    json: bool,

    /// Specify the table name for JSON output.
    #[arg(short = 'n', long = "table-name", value_name = "name")]
    table_name: Option<String>,

    /// Specify column to use for tree-like output.
    #[arg(short = 'r', long = "tree", value_name = "column")]
    tree: Option<String>,

    /// Specify column containing line IDs for tree parent-child relations.
    #[arg(short = 'i', long = "tree-id", value_name = "column")]
    tree_id: Option<String>,

    /// Specify column containing parent IDs for tree parent-child relations.
    #[arg(short = 'p', long = "tree-parent", value_name = "column")]
    tree_parent: Option<String>,

    /// Characters that separate input items (whitespace by default).
    /// --input-separator is accepted as an alias, as in upstream
    /// column(1).
    #[arg(
        short,
        long,
        visible_alias = "input-separator",
        value_name = "separators"
    )]
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

    /// Right align text in specified columns.
    #[arg(short = 'R', long = "table-right", value_name = "columns")]
    table_right: Option<String>,

    /// Truncate text in specified columns when necessary.
    #[arg(short = 'T', long = "table-truncate", value_name = "columns")]
    table_truncate: Option<String>,

    /// Ignore unusually long cells when calculating column width.
    #[arg(short = 'E', long = "table-noextreme", value_name = "columns")]
    table_noextreme: Option<String>,

    /// Allow multi-line wrapping in specified columns.
    #[arg(short = 'W', long = "table-wrap", value_name = "columns")]
    table_wrap: Option<String>,

    /// Don't print specified columns.
    #[arg(short = 'H', long = "table-hide", value_name = "columns")]
    table_hide: Option<String>,

    /// Specify output column order.
    #[arg(short = 'O', long = "table-order", value_name = "columns")]
    table_order: Option<String>,

    /// Print header line for each page.
    #[arg(short = 'e', long = "table-header-repeat")]
    table_header_repeat: bool,

    /// Define a column with attributes (can be repeated).
    #[arg(short = 'C', long = "table-column", value_name = "attributes")]
    table_column: Vec<String>,

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

/// Terminal width for layout decisions, or `None` when stdout
/// isn't a TTY. Upstream `column(1)` only constrains column widths
/// to the terminal width when running interactively — piping to a
/// file should let columns grow to fit their content.
fn terminal_width() -> Option<usize> {
    let stdout = io::stdout();
    match rustix::termios::tcgetwinsize(&stdout) {
        Ok(ws) if ws.ws_col > 0 => Some(ws.ws_col as usize),
        _ => None,
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
    let width_limit: Option<usize> = match &args.output_width {
        Some(s) => parse_output_width(s),
        None => terminal_width(),
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

    let result = if args.table || args.json || args.tree.is_some() {
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
    use cols::{OutputMode, Table, TermForce, print_table};
    use std::collections::HashMap;

    // Parse input into rows of fields.
    let rows = parse_table_input(
        input,
        args.separator.as_deref(),
        args.table_columns_limit,
        args.keep_empty_lines,
    );

    if rows.is_empty() {
        return Ok(());
    }

    // Determine column count from the widest row, but always
    // honour `--table-columns` if it lists more names than the
    // data rows actually have. Upstream renders the extra columns
    // as empty so headers like ID, PARENT, … stay visible even
    // when `--table-columns-limit` collapsed everything past the
    // limit into the last data column.
    let data_ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let user_name_ncols = args
        .table_columns
        .as_ref()
        .map_or(0, |s| s.split(',').count());
    let ncols = data_ncols.max(user_name_ncols);
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
        table.termwidth_set(width);
        table.termforce_set(TermForce::Always);
    }

    // Hide headings when the user asked for it, or when they
    // didn't provide column names via either `-N`/`--table-columns`
    // or `-C`/`--table-column name=…` (in which case we'd be
    // showing synthesised `COL1, COL2, …` placeholders).
    let any_named_via_c = args
        .table_column
        .iter()
        .any(|attr| parse_column_attrs(attr).iter().any(|(k, _)| k == "name"));
    if args.table_noheadings
        || (args.table_columns.is_none() && !any_named_via_c)
    {
        table.headings_set(false);
    }

    // Upstream `column --table` uses ASCII tree drawing
    // (`|-`, `` `- ``) rather than the Unicode box characters.
    table.ascii_set(true);

    if args.table_maxout {
        table.maxout_set(true);
    }

    // Upstream `column --table` uses two spaces between columns.
    // The cols crate's default is one space, so set the default
    // here unless the user overrides with `-o`.
    if let Some(ref sep) = args.output_separator {
        table.column_separator_set(sep);
    } else {
        table.column_separator_set("  ");
    }

    // The cols crate uses each column's header text as a *minimum*
    // width when computing layout, even when headings are turned
    // off. That doesn't match upstream `column(1)`, where the
    // synthesised `COL1, COL2, …` placeholders or
    // `--table-noheadings` should leave layout driven purely by
    // data widths. Pass empty names to cols in either case (JSON
    // mode keeps the real names so its keys are still meaningful).
    let suppress_header_widths = !args.json
        && (args.table_noheadings
            || (args.table_columns.is_none() && !any_named_via_c));
    let layout_names: Vec<String> = if suppress_header_widths {
        col_names.iter().map(|_| String::new()).collect()
    } else {
        col_names.clone()
    };

    // `--table-order` chooses the *display* order. Build a
    // permutation `display_order[d] = orig` mapping each display
    // position to the original column index, with any unlisted
    // columns appended in their natural order. The cols crate
    // doesn't reorder columns after creation, so we create them
    // up-front in the desired order and remap data accordingly.
    let display_order: Vec<usize> = if let Some(ref spec) = args.table_order {
        let mut ordered = Vec::with_capacity(col_names.len());
        let mut seen = vec![false; col_names.len()];
        for idx in resolve_columns(spec, &col_names) {
            if !seen[idx] {
                ordered.push(idx);
                seen[idx] = true;
            }
        }
        for (i, &was_seen) in seen.iter().enumerate() {
            if !was_seen {
                ordered.push(i);
            }
        }
        ordered
    } else {
        (0..col_names.len()).collect()
    };
    let mut orig_to_display = vec![None; col_names.len()];
    for (d, &o) in display_order.iter().enumerate() {
        orig_to_display[o] = Some(d);
    }

    // Create columns in display order. `-C` per-column attribute
    // strings still apply by *original* index, so we look up the
    // matching attribute set when creating each column.
    for &orig in &display_order {
        let name = layout_names.get(orig).map(|s| s.as_str()).unwrap_or("");
        let idx = table.new_column(name);
        if let Some(attr_str) = args.table_column.get(orig) {
            let attrs = parse_column_attrs(attr_str);
            // `name=...` from `-C` overrides the default header.
            if let Some(custom_name) = attrs.iter().find_map(|(k, v)| {
                if k == "name" { v.as_deref() } else { None }
            }) {
                table.column_mut(idx).unwrap().name_set(custom_name);
            }
            let col = table.column_mut(idx).unwrap();
            for (key, val) in &attrs {
                match key.as_str() {
                    "right" => {
                        col.right_set(true);
                    }
                    "trunc" => {
                        col.truncate_set(true);
                    }
                    "noextreme" => {
                        col.no_extremes_set(true);
                    }
                    "wrap" => {
                        col.wrap_set(true);
                    }
                    "hide" => {
                        col.hidden_set(true);
                    }
                    "strictwidth" => {
                        col.strict_width_set(true);
                    }
                    "width" => {
                        if let Some(v) = val.as_deref()
                            && let Ok(w) = v.parse::<usize>()
                        {
                            col.width_hint_set(cols::WidthHint::Fixed(w));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Apply column flags. `resolve_columns` returns *original*
    // indices; remap each to the display position before mutating
    // the cols `Column`.
    let apply_flag =
        |table: &mut Table, spec: &str, f: &dyn Fn(&mut cols::Column)| {
            for orig in resolve_columns(spec, &col_names) {
                if let Some(d) = orig_to_display[orig]
                    && let Some(col) = table.column_mut(d)
                {
                    f(col);
                }
            }
        };
    if let Some(ref spec) = args.table_right {
        apply_flag(&mut table, spec, &|c| {
            c.right_set(true);
        });
    }
    if let Some(ref spec) = args.table_truncate {
        apply_flag(&mut table, spec, &|c| {
            c.truncate_set(true);
        });
    }
    if let Some(ref spec) = args.table_noextreme {
        apply_flag(&mut table, spec, &|c| {
            c.no_extremes_set(true);
        });
    }
    if let Some(ref spec) = args.table_wrap {
        apply_flag(&mut table, spec, &|c| {
            c.wrap_set(true);
        });
    }
    if let Some(ref spec) = args.table_hide {
        apply_flag(&mut table, spec, &|c| {
            c.hidden_set(true);
        });
    }

    if args.table_header_repeat {
        table.header_repeat_set(true);
    }

    // JSON mode.
    if args.json {
        table.output_mode_set(OutputMode::Json);
        if let Some(ref name) = args.table_name {
            table.name_set(name);
        } else {
            table.name_set("table");
        }
    }

    // Resolve tree column indices. These are the *original* column
    // indices; the tree-set call wants the display position.
    let tree_col = args
        .tree
        .as_ref()
        .and_then(|s| resolve_single_column(s, &col_names));
    let tree_id_col = args
        .tree_id
        .as_ref()
        .and_then(|s| resolve_single_column(s, &col_names));
    let tree_parent_col = args
        .tree_parent
        .as_ref()
        .and_then(|s| resolve_single_column(s, &col_names));

    // Mark the tree column at its display position.
    if let Some(tc) = tree_col
        && let Some(d) = orig_to_display[tc]
        && let Some(col) = table.column_mut(d)
    {
        col.tree_set(true);
    }

    // Populate rows. Each cell is sourced by the *original* column
    // index, then written at the matching *display* position.
    let set_row = |line: &mut cols::Line, row: &[String]| {
        for (d, &orig) in display_order.iter().enumerate() {
            if let Some(cell) = row.get(orig) {
                line.data_set(d, cell);
            }
        }
    };

    if let (Some(id_col), Some(parent_col)) = (tree_id_col, tree_parent_col) {
        // Tree mode: build parent-child relationships from data.
        let mut id_to_line: HashMap<String, cols::LineId> = HashMap::new();
        let mut deferred_parents: Vec<(cols::LineId, String)> = Vec::new();

        for row in &rows {
            let line_id = table.new_line(None);
            set_row(table.line_mut(line_id), row);
            if let Some(id_val) = row.get(id_col) {
                id_to_line.insert(id_val.clone(), line_id);
            }
            if let Some(parent_val) = row.get(parent_col)
                && !parent_val.is_empty()
                && parent_val != "0"
            {
                deferred_parents.push((line_id, parent_val.clone()));
            }
        }

        // Wire up parent-child relationships.
        for (child_line, parent_id) in &deferred_parents {
            if let Some(&parent_line) = id_to_line.get(parent_id) {
                table.add_child(parent_line, *child_line);
            }
        }
    } else {
        for row in &rows {
            let line_id = table.new_line(None);
            set_row(table.line_mut(line_id), row);
        }
    }

    print_table(&table, writer)
}

/// Resolve a column specification string to 0-based column indices.
///
/// The spec is a comma-separated list of:
/// - `0` — all columns
/// - `-1` — last column
/// - `N` — 1-based index
/// - `N-M` — range of 1-based indices
/// - `name` — column name (matched against `col_names`)
fn resolve_columns(spec: &str, col_names: &[String]) -> Vec<usize> {
    let ncols = col_names.len();
    let mut result = Vec::new();

    for part in spec.split(',') {
        let part = part.trim();
        if part == "0" {
            result.extend(0..ncols);
        } else if part == "-" {
            // Special: all unnamed columns (auto-generated COLn names).
            for (i, name) in col_names.iter().enumerate() {
                if name.starts_with("COL") && name[3..].parse::<usize>().is_ok()
                {
                    result.push(i);
                }
            }
        } else if let Some(rest) = part.strip_prefix('-')
            && let Ok(n) = rest.parse::<usize>()
            && n >= 1
            && n <= ncols
        {
            // Negative index: `-1` = last column, `-N` = N-from-last.
            result.push(ncols - n);
        } else if let Ok(n) = part.parse::<usize>() {
            if n >= 1 && n <= ncols {
                result.push(n - 1);
            }
        } else if let Some(dash_pos) = part.find('-')
            && dash_pos > 0
            && let (Ok(start), Ok(end)) = (
                part[..dash_pos].parse::<usize>(),
                part[dash_pos + 1..].parse::<usize>(),
            )
            && start >= 1
            && end >= start
        {
            for i in start..=end.min(ncols) {
                result.push(i - 1);
            }
        } else {
            // Match by name. Names with embedded `-` (e.g.
            // `VFS-OPTS`) end up here because the integer-range
            // attempt above fell through.
            for (i, name) in col_names.iter().enumerate() {
                if name == part {
                    result.push(i);
                }
            }
        }
    }

    result
}

/// Parse `-C` column attribute string like "name=FOO,right,trunc".
fn parse_column_attrs(s: &str) -> Vec<(String, Option<String>)> {
    s.split(',')
        .map(|attr| {
            if let Some((k, v)) = attr.split_once('=') {
                (k.trim().to_string(), Some(v.trim().to_string()))
            } else {
                (attr.trim().to_string(), None)
            }
        })
        .collect()
}

/// Resolve a single column reference to a 0-based index.
fn resolve_single_column(spec: &str, col_names: &[String]) -> Option<usize> {
    resolve_columns(spec, col_names).into_iter().next()
}

/// Parse input text into rows of fields based on separator.
fn parse_table_input(
    input: &str,
    separator: Option<&str>,
    columns_limit: Option<usize>,
    keep_empty: bool,
) -> Vec<Vec<String>> {
    let mut rows = Vec::new();

    for line in input.lines() {
        if line.is_empty() {
            if keep_empty {
                rows.push(Vec::new());
            }
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
