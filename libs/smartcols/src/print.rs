use crate::{
    column::JsonType,
    line::LineId,
    table::{OutputMode, Table, TermForce},
};
use std::{
    fmt::Write as _,
    io::{self, Write},
};
use unicode_width::UnicodeWidthStr;

/// Print the table to a writer.
pub fn print_table(table: &Table, out: &mut dyn Write) -> io::Result<()> {
    match table.output_mode() {
        OutputMode::Normal => print_normal(table, out),
        OutputMode::Raw => print_raw(table, out),
        OutputMode::Export => print_export(table, out),
        OutputMode::Json => print_json(table, out),
        OutputMode::ShellVar => print_shellvar(table, out),
    }
}

/// Render the table to a String.
pub fn print_table_to_string(table: &Table) -> io::Result<String> {
    let mut buf = Vec::new();
    print_table(table, &mut buf)?;
    String::from_utf8(buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn print_normal(table: &Table, out: &mut dyn Write) -> io::Result<()> {
    let visible: Vec<usize> = visible_columns(table);
    if visible.is_empty() {
        return Ok(());
    }

    let termwidth = effective_termwidth(table);
    let widths = calculate_widths(table, &visible, termwidth);

    // Title
    if table.title().data().is_some() {
        print_title(table, out, termwidth)?;
    }

    // Header
    if !table.is_no_headings() {
        print_normal_header(table, out, &visible, &widths)?;
    }

    // Lines — walk in tree order if any column has the TREE flag.
    if table.is_tree() {
        let roots = table.root_line_ids();
        for root in roots {
            print_normal_tree_line(
                table,
                out,
                root,
                &visible,
                &widths,
                &[],
                0,
            )?;
        }
    } else {
        for id in table.line_ids() {
            print_normal_line(table, out, id, &visible, &widths, "")?;
        }
    }

    Ok(())
}

fn print_title(
    table: &Table,
    out: &mut dyn Write,
    termwidth: usize,
) -> io::Result<()> {
    if let Some(data) = table.title().data() {
        let pad_char = &table.symbols().title_padding;
        let data_width = display_width(data);
        if data_width < termwidth {
            let total_pad = termwidth - data_width;
            let left = total_pad / 2;
            let right = total_pad - left;
            for _ in 0..left {
                write!(out, "{pad_char}")?;
            }
            write!(out, "{data}")?;
            for _ in 0..right {
                write!(out, "{pad_char}")?;
            }
        } else {
            write!(out, "{data}")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn print_normal_header(
    table: &Table,
    out: &mut dyn Write,
    visible: &[usize],
    widths: &[usize],
) -> io::Result<()> {
    for (vi, &col_idx) in visible.iter().enumerate() {
        let col = &table.columns()[col_idx];
        let header_text = col.header().data().unwrap_or("");
        let width = widths[vi];
        let is_last = vi == visible.len() - 1;

        if vi > 0 {
            write!(out, "{}", table.column_separator())?;
        }

        if is_last && !table.is_maxout() {
            // Last column: no trailing padding unless maxout.
            write!(out, "{header_text}")?;
        } else if col.is_right() {
            write!(out, "{header_text:>width$}")?;
        } else {
            write!(out, "{header_text:<width$}")?;
        }
    }
    writeln!(out)?;
    Ok(())
}

fn print_normal_line(
    table: &Table,
    out: &mut dyn Write,
    id: LineId,
    visible: &[usize],
    widths: &[usize],
    tree_prefix: &str,
) -> io::Result<()> {
    let line = table.line(id);

    for (vi, &col_idx) in visible.iter().enumerate() {
        let col = &table.columns()[col_idx];
        let raw_data = line.get_data(col_idx).unwrap_or("");
        let is_last = vi == visible.len() - 1;
        let width = widths[vi];

        if vi > 0 {
            write!(out, "{}", table.column_separator())?;
        }

        // Encode non-printable characters unless disabled.
        let data = if table.is_no_encoding() {
            raw_data.to_string()
        } else {
            encode_data(raw_data, col.safechars())
        };

        // Prepend tree prefix for the tree column.
        let data = if col.is_tree() && !tree_prefix.is_empty() {
            format!("{tree_prefix}{data}")
        } else {
            data
        };

        let data_width = display_width(&data);

        if is_last && !table.is_maxout() {
            // Last column: no trailing padding unless maxout.
            if table.is_minout() && data.is_empty() {
                // minout: skip trailing empty columns
            } else {
                write!(out, "{data}")?;
            }
        } else if col.is_right() {
            if data_width < width {
                let pad = width - data_width;
                for _ in 0..pad {
                    write!(out, " ")?;
                }
            }
            write!(out, "{data}")?;
        } else if col.is_trunc() && data_width > width {
            write!(out, "{}", truncate_to_width(&data, width))?;
        } else {
            write!(out, "{data}")?;
            if data_width < width {
                let pad = width - data_width;
                for _ in 0..pad {
                    write!(out, " ")?;
                }
            }
        }
    }

    if !table.is_no_linesep() {
        write!(out, "{}", table.line_separator())?;
    }
    Ok(())
}

fn print_normal_tree_line(
    table: &Table,
    out: &mut dyn Write,
    id: LineId,
    visible: &[usize],
    widths: &[usize],
    ancestors_last: &[bool],
    depth: usize,
) -> io::Result<()> {
    let line = table.line(id);
    let children = line.children().to_vec();
    let is_last_child = ancestors_last.last().copied().unwrap_or(true);

    // Build tree prefix for this line.
    let prefix = if depth == 0 {
        String::new()
    } else {
        let symbols = table.symbols();
        let mut pfx = String::new();
        // Draw vertical lines for ancestors that are not the last child.
        for &ancestor_is_last in &ancestors_last[..ancestors_last.len() - 1] {
            if ancestor_is_last {
                pfx.push_str(&symbols.cell_padding);
                pfx.push_str(&symbols.cell_padding);
            } else {
                pfx.push_str(&symbols.vertical);
            }
        }
        // Draw the branch or right connector.
        if is_last_child {
            pfx.push_str(&symbols.right);
        } else {
            pfx.push_str(&symbols.branch);
        }
        pfx
    };

    print_normal_line(table, out, id, visible, widths, &prefix)?;

    // Recurse into children.
    for (i, &child) in children.iter().enumerate() {
        let mut new_ancestors = ancestors_last.to_vec();
        new_ancestors.push(i == children.len() - 1);
        print_normal_tree_line(
            table,
            out,
            child,
            visible,
            widths,
            &new_ancestors,
            depth + 1,
        )?;
    }

    Ok(())
}

fn print_raw(table: &Table, out: &mut dyn Write) -> io::Result<()> {
    let visible = visible_columns(table);
    if visible.is_empty() {
        return Ok(());
    }

    if !table.is_no_headings() {
        for (vi, &col_idx) in visible.iter().enumerate() {
            if vi > 0 {
                write!(out, "{}", table.column_separator())?;
            }
            let header = table.columns()[col_idx].header().data().unwrap_or("");
            write!(out, "{header}")?;
        }
        writeln!(out)?;
    }

    for id in table.line_ids() {
        let line = table.line(id);
        for (vi, &col_idx) in visible.iter().enumerate() {
            if vi > 0 {
                write!(out, "{}", table.column_separator())?;
            }
            let data = line.get_data(col_idx).unwrap_or("");
            write!(out, "{data}")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn print_export(table: &Table, out: &mut dyn Write) -> io::Result<()> {
    let visible = visible_columns(table);

    for id in table.line_ids() {
        let line = table.line(id);
        for (vi, &col_idx) in visible.iter().enumerate() {
            let col = &table.columns()[col_idx];
            let name = col
                .name_as_shellvar()
                .unwrap_or_else(|| format!("COL{col_idx}"));
            let data = line.get_data(col_idx).unwrap_or("");
            if vi > 0 {
                write!(out, " ")?;
            }
            write!(out, "{name}=\"{}\"", escape_export(data))?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn escape_export(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn print_shellvar(table: &Table, out: &mut dyn Write) -> io::Result<()> {
    // Same format as export but with shell-safe column names.
    print_export(table, out)
}

fn print_json(table: &Table, out: &mut dyn Write) -> io::Result<()> {
    let visible = visible_columns(table);
    let table_name = table.name().unwrap_or("table");

    writeln!(out, "{{")?;
    writeln!(out, "   \"{table_name}\": [")?;

    let line_ids: Vec<LineId> = table.line_ids().collect();
    for (li, &id) in line_ids.iter().enumerate() {
        let line = table.line(id);
        write!(out, "      {{")?;

        for (vi, &col_idx) in visible.iter().enumerate() {
            let col = &table.columns()[col_idx];
            let name = col.name().unwrap_or("unknown");
            let data = line.get_data(col_idx).unwrap_or("");

            if vi > 0 {
                write!(out, ",")?;
            }
            writeln!(out)?;
            write!(out, "         ")?;

            match col.json_type() {
                JsonType::Number => {
                    let num = data.parse::<i64>().unwrap_or(0);
                    write!(out, "\"{name}\": {num}")?;
                }
                JsonType::Float => {
                    let num = data.parse::<f64>().unwrap_or(0.0);
                    write!(out, "\"{name}\": {num}")?;
                }
                JsonType::Boolean => {
                    let b = matches!(data, "1" | "true" | "TRUE" | "yes");
                    write!(out, "\"{name}\": {b}")?;
                }
                JsonType::BooleanOptional => {
                    if data.is_empty() {
                        write!(out, "\"{name}\": null")?;
                    } else {
                        let b = matches!(data, "1" | "true" | "TRUE" | "yes");
                        write!(out, "\"{name}\": {b}")?;
                    }
                }
                JsonType::ArrayString => {
                    let items: Vec<&str> = data.lines().collect();
                    write!(out, "\"{name}\": [")?;
                    for (ai, item) in items.iter().enumerate() {
                        if ai > 0 {
                            write!(out, ", ")?;
                        }
                        write!(out, "\"{}\"", escape_json(item))?;
                    }
                    write!(out, "]")?;
                }
                JsonType::ArrayNumber => {
                    let items: Vec<&str> = data.lines().collect();
                    write!(out, "\"{name}\": [")?;
                    for (ai, item) in items.iter().enumerate() {
                        if ai > 0 {
                            write!(out, ", ")?;
                        }
                        let num = item.trim().parse::<i64>().unwrap_or(0);
                        write!(out, "{num}")?;
                    }
                    write!(out, "]")?;
                }
                JsonType::String => {
                    write!(out, "\"{name}\": \"{}\"", escape_json(data))?;
                }
            }
        }

        writeln!(out)?;
        if li < line_ids.len() - 1 {
            writeln!(out, "      }},")?;
        } else {
            writeln!(out, "      }}")?;
        }
    }

    writeln!(out, "   ]")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn visible_columns(table: &Table) -> Vec<usize> {
    table
        .columns()
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_hidden())
        .map(|(i, _)| i)
        .collect()
}

fn calculate_widths(
    table: &Table,
    visible: &[usize],
    termwidth: usize,
) -> Vec<usize> {
    let ncols = visible.len();
    if ncols == 0 {
        return vec![];
    }

    let sep_width = display_width(table.column_separator());
    let total_sep = sep_width * ncols.saturating_sub(1);
    let available = termwidth.saturating_sub(total_sep);

    // Collect per-column metrics.
    #[allow(dead_code)]
    struct ColInfo {
        header_w: usize,
        max_data_w: usize,
        hint: f64,
        is_trunc: bool,
        is_strict: bool,
        is_noextremes: bool,
    }

    // Pre-compute tree depth for each line (needed for tree column width).
    let tree_symbol_width = display_width(&table.symbols().branch);
    let tree_depths: Vec<usize> = table
        .line_ids()
        .map(|id| {
            let mut depth = 0;
            let mut cur = table.line(id).parent();
            while let Some(pid) = cur {
                depth += 1;
                cur = table.line(pid).parent();
            }
            depth
        })
        .collect();

    let infos: Vec<ColInfo> = visible
        .iter()
        .map(|&col_idx| {
            let col = &table.columns()[col_idx];
            let header_w = col.header().data().map_or(0, display_width);
            let max_data_w = table
                .line_ids()
                .enumerate()
                .map(|(li, id)| {
                    let data_w = table
                        .line(id)
                        .get_data(col_idx)
                        .map_or(0, display_width);
                    if col.is_tree() && tree_depths[li] > 0 {
                        data_w + tree_depths[li] * tree_symbol_width
                    } else {
                        data_w
                    }
                })
                .max()
                .unwrap_or(0);
            ColInfo {
                header_w,
                max_data_w,
                hint: col.whint(),
                is_trunc: col.is_trunc(),
                is_strict: col.is_strict_width(),
                is_noextremes: col.is_no_extremes(),
            }
        })
        .collect();

    // Phase 1: compute each column's desired width from its hint.
    // Hints are starting points, not final widths.
    let mut widths: Vec<usize> = infos
        .iter()
        .map(|info| {
            let hint_w = if info.hint >= 1.0 {
                info.hint as usize
            } else if info.hint > 0.0 {
                (available as f64 * info.hint) as usize
            } else {
                0
            };
            // At minimum, fit the header.
            info.header_w.max(hint_w)
        })
        .collect();

    if !use_terminal_width(table) {
        // No terminal constraint: use natural content widths.
        for (i, info) in infos.iter().enumerate() {
            widths[i] = widths[i].max(info.max_data_w);
        }
        return widths;
    }

    // Phase 2: distribute remaining space to columns that need more.
    // Give each column up to its max_data_w, distributing fairly.
    loop {
        let total: usize = widths.iter().sum();
        let remaining = available.saturating_sub(total);
        if remaining == 0 {
            break;
        }

        // Find columns that want more space (data wider than current width).
        let mut grew = false;
        // Give space to columns with the biggest deficit first.
        let mut deficits: Vec<(usize, usize)> = infos
            .iter()
            .enumerate()
            .filter_map(|(i, info)| {
                let wanted = info.max_data_w;
                if wanted > widths[i] {
                    Some((i, wanted - widths[i]))
                } else {
                    None
                }
            })
            .collect();
        deficits.sort_by(|a, b| b.1.cmp(&a.1));

        for (i, _) in &deficits {
            let total: usize = widths.iter().sum();
            if total >= available {
                break;
            }
            let can_give = available - total;
            let want = infos[*i].max_data_w.saturating_sub(widths[*i]);
            let give = want.min(can_give);
            if give > 0 {
                widths[*i] += give;
                grew = true;
            }
        }

        if !grew {
            break;
        }
    }

    // Phase 3: if still too wide, shrink.
    let total: usize = widths.iter().sum();
    if total > available {
        // Iteratively shrink the widest non-strict column.
        let mut remaining =
            widths.iter().sum::<usize>().saturating_sub(available);
        while remaining > 0 {
            let candidate = widths
                .iter()
                .enumerate()
                .filter(|(vi, w)| {
                    !infos[*vi].is_strict && **w > infos[*vi].header_w.max(1)
                })
                .max_by_key(|(_, w)| **w)
                .map(|(i, _)| i);

            if let Some(i) = candidate {
                widths[i] -= 1;
                remaining -= 1;
            } else {
                break;
            }
        }
    } else if table.is_maxout() && total < available {
        let extra = available - total;
        let per_col = extra / ncols;
        let leftover = extra % ncols;
        for (i, w) in widths.iter_mut().enumerate() {
            *w += per_col;
            if i < leftover {
                *w += 1;
            }
        }
    }

    widths
}

fn use_terminal_width(table: &Table) -> bool {
    match table.termforce() {
        TermForce::Always => true,
        TermForce::Never => false,
        TermForce::Auto => true, // default: respect terminal width
    }
}

fn effective_termwidth(table: &Table) -> usize {
    let w = table.termwidth();
    w.saturating_sub(0) // placeholder for reduce; table already has reduce_termwidth
}

/// Visible (display) width of a string, accounting for Unicode.
fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Truncate a string to fit in `max_width` display columns.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut result = String::new();
    for c in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + cw > max_width {
            break;
        }
        width += cw;
        result.push(c);
    }
    result
}

/// Encode non-printable characters as `\xHH`.
fn encode_data(s: &str, safechars: Option<&str>) -> String {
    let safe = safechars.unwrap_or("");
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_control() && c != '\t' && !safe.contains(c) {
            for b in c.to_string().as_bytes() {
                let _ = write!(out, "\\x{b:02x}");
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColumnFlags, Table};

    #[test]
    fn basic_normal_output() {
        let mut table = Table::new();
        table.new_column("NAME", 0.0, ColumnFlags::empty());
        table.new_column("SIZE", 0.0, ColumnFlags::RIGHT);

        let l1 = table.new_line(None);
        table.line_mut(l1).set_data(0, "sda").set_data(1, "100G");

        let l2 = table.new_line(None);
        table.line_mut(l2).set_data(0, "sdb").set_data(1, "50G");

        let output = print_table_to_string(&table).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 data lines
        assert!(lines[0].starts_with("NAME"));
        assert!(lines[1].contains("sda"));
        assert!(lines[2].contains("sdb"));
    }

    #[test]
    fn no_headings() {
        let mut table = Table::new();
        table.enable_no_headings(true);
        table.new_column("NAME", 0.0, ColumnFlags::empty());

        let l1 = table.new_line(None);
        table.line_mut(l1).set_data(0, "hello");

        let output = print_table_to_string(&table).unwrap();
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn json_output() {
        let mut table = Table::new();
        table.set_name("devices");
        table.set_output_mode(OutputMode::Json);
        table.new_column("NAME", 0.0, ColumnFlags::empty());

        let l1 = table.new_line(None);
        table.line_mut(l1).set_data(0, "sda");

        let output = print_table_to_string(&table).unwrap();
        assert!(output.contains("\"devices\""));
        assert!(output.contains("\"NAME\": \"sda\""));
    }

    #[test]
    fn export_output() {
        let mut table = Table::new();
        table.set_output_mode(OutputMode::Export);
        table.new_column("NAME", 0.0, ColumnFlags::empty());
        table.new_column("SIZE", 0.0, ColumnFlags::empty());

        let l1 = table.new_line(None);
        table.line_mut(l1).set_data(0, "sda").set_data(1, "100G");

        let output = print_table_to_string(&table).unwrap();
        assert!(output.contains("NAME=\"sda\""));
        assert!(output.contains("SIZE=\"100G\""));
    }

    #[test]
    fn raw_output() {
        let mut table = Table::new();
        table.set_output_mode(OutputMode::Raw);
        table.new_column("A", 0.0, ColumnFlags::empty());
        table.new_column("B", 0.0, ColumnFlags::empty());

        let l1 = table.new_line(None);
        table.line_mut(l1).set_data(0, "x").set_data(1, "y");

        let output = print_table_to_string(&table).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[0], "A B");
        assert_eq!(lines[1], "x y");
    }

    #[test]
    fn tree_output() {
        let mut table = Table::new();
        table.enable_no_headings(true);
        table.enable_ascii(true);
        table.new_column("NAME", 0.0, ColumnFlags::TREE);

        let root = table.new_line(None);
        table.line_mut(root).set_data(0, "root");

        let child1 = table.new_line(Some(root));
        table.line_mut(child1).set_data(0, "child1");

        let child2 = table.new_line(Some(root));
        table.line_mut(child2).set_data(0, "child2");

        let output = print_table_to_string(&table).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[0], "root");
        assert_eq!(lines[1], "|-child1");
        assert_eq!(lines[2], "`-child2");
    }

    #[test]
    fn hidden_column() {
        let mut table = Table::new();
        table.enable_no_headings(true);
        table.new_column("VISIBLE", 0.0, ColumnFlags::empty());
        table.new_column("HIDDEN", 0.0, ColumnFlags::HIDDEN);

        let l1 = table.new_line(None);
        table.line_mut(l1).set_data(0, "yes").set_data(1, "no");

        let output = print_table_to_string(&table).unwrap();
        assert!(output.contains("yes"));
        assert!(!output.contains("no"));
    }

    #[test]
    fn encode_nonprintable() {
        let encoded = encode_data("hello\x01world", None);
        assert_eq!(encoded, "hello\\x01world");
    }

    #[test]
    fn json_escaping() {
        let escaped = escape_json("hello \"world\"\nnewline");
        assert_eq!(escaped, "hello \\\"world\\\"\\nnewline");
    }
}
