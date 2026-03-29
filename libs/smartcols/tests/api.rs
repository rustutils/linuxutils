use smartcols::{
    Alignment, Cell, Column, DataType, Filter, JsonType, Line, OutputMode,
    Symbols, Table, TermForce, print_table_to_string,
};

// ── Cell ─────────────────────────────────────────────────────────────────────

#[test]
fn cell_default_is_empty() {
    let c = Cell::new();
    assert!(c.data().is_none());
    assert!(!c.is_filled());
}

#[test]
fn cell_set_and_clear() {
    let mut c = Cell::new();
    c.set_data("hello");
    assert_eq!(c.data(), Some("hello"));
    assert!(c.is_filled());
    c.clear_data();
    assert!(c.data().is_none());
}

#[test]
fn cell_alignment() {
    let mut c = Cell::new();
    assert_eq!(c.alignment(), Alignment::Left);
    c.set_alignment(Alignment::Right);
    assert_eq!(c.alignment(), Alignment::Right);
    c.set_alignment(Alignment::Center);
    assert_eq!(c.alignment(), Alignment::Center);
    c.set_alignment(Alignment::Left);
    assert_eq!(c.alignment(), Alignment::Left);
}

#[test]
fn cell_color_and_uri() {
    let mut c = Cell::new();
    assert!(c.color().is_none());
    assert!(c.uri().is_none());
    c.set_color("red");
    c.set_uri("https://example.com");
    assert_eq!(c.color(), Some("red"));
    assert_eq!(c.uri(), Some("https://example.com"));
}

#[test]
fn cell_reset() {
    let mut c = Cell::new();
    c.set_data("x").set_color("blue");
    c.reset();
    assert!(c.data().is_none());
    assert!(c.color().is_none());
}

#[test]
fn cell_clone() {
    let mut c = Cell::new();
    c.set_data("original");
    let mut clone = c.clone();
    clone.set_data("modified");
    assert_eq!(c.data(), Some("original"));
    assert_eq!(clone.data(), Some("modified"));
}

// ── Column ───────────────────────────────────────────────────────────────────

#[test]
fn column_defaults() {
    let c = Column::new();
    assert!(c.name().is_none());
    assert_eq!(c.whint(), 0.0);
    assert!(!c.is_tree());
    assert!(!c.is_right());
    assert_eq!(c.json_type(), JsonType::String);
    assert_eq!(c.data_type(), DataType::None);
}

#[test]
fn column_with_name() {
    let mut c = Column::with_name("SIZE", 6.0);
    c.set_right(true);
    assert_eq!(c.name(), Some("SIZE"));
    assert_eq!(c.whint(), 6.0);
    assert!(c.is_right());
    assert_eq!(c.header().data(), Some("SIZE"));
}

#[test]
fn column_flags() {
    let mut c = Column::with_name("x", 0.0);
    c.set_tree(true).set_trunc(true);
    assert!(c.is_tree());
    assert!(c.is_trunc());
    assert!(!c.is_right());
    assert!(!c.is_hidden());
    assert!(!c.is_wrap());
}

#[test]
fn column_name_as_shellvar() {
    let mut c = Column::new();
    c.set_name("MAJ:MIN");
    assert_eq!(c.name_as_shellvar(), Some("MAJ_MIN".to_string()));
    c.set_name("NORMAL");
    assert_eq!(c.name_as_shellvar(), Some("NORMAL".to_string()));
}

#[test]
fn column_json_and_data_types() {
    let mut c = Column::new();
    c.set_json_type(JsonType::Number);
    assert_eq!(c.json_type(), JsonType::Number);
    c.set_data_type(DataType::U64);
    assert_eq!(c.data_type(), DataType::U64);
}

#[test]
fn column_clone_shares_cmp_func() {
    let mut c = Column::new();
    c.set_name("N");
    c.set_cmp_func(|a, b| {
        let av: i64 = a.data().unwrap_or("0").parse().unwrap_or(0);
        let bv: i64 = b.data().unwrap_or("0").parse().unwrap_or(0);
        av.cmp(&bv)
    });

    let clone = c.clone();
    let mut ca = Cell::new();
    ca.set_data("10");
    let mut cb = Cell::new();
    cb.set_data("3");
    // Both original and clone should compare numerically (10 > 3).
    assert_eq!(c.cmp_cells(&ca, &cb), std::cmp::Ordering::Greater);
    assert_eq!(clone.cmp_cells(&ca, &cb), std::cmp::Ordering::Greater);
}

// ── Line ─────────────────────────────────────────────────────────────────────

#[test]
fn line_set_and_get_data() {
    let mut l = Line::new();
    l.set_data(0, "hello").set_data(1, "world");
    assert_eq!(l.get_data(0), Some("hello"));
    assert_eq!(l.get_data(1), Some("world"));
    assert!(l.get_data(2).is_none());
    assert_eq!(l.ncells(), 2);
}

#[test]
fn line_alloc_cells() {
    let mut l = Line::new();
    l.alloc_cells(5);
    assert_eq!(l.ncells(), 5);
    assert!(!l.is_filled(0));
}

#[test]
fn line_clone() {
    let mut l = Line::new();
    l.set_data(0, "original");
    let mut clone = l.clone();
    clone.set_data(0, "modified");
    assert_eq!(l.get_data(0), Some("original"));
}

// ── Table construction ────────────────────────────────────────────────────────

#[test]
fn table_new_column_returns_index() {
    let mut t = Table::new();
    let i0 = t.new_column("A", 0.0);
    let i1 = t.new_column("B", 0.0);
    assert_eq!(i0, 0);
    assert_eq!(i1, 1);
    assert_eq!(t.ncols(), 2);
}

#[test]
fn table_add_and_remove_column() {
    let mut t = Table::new();
    t.new_column("A", 0.0);
    t.new_column("B", 0.0);
    let removed = t.remove_column(0);
    assert_eq!(removed.name(), Some("A"));
    assert_eq!(t.ncols(), 1);
    assert_eq!(t.columns()[0].name(), Some("B"));
}

#[test]
fn table_column_by_name() {
    let mut t = Table::new();
    t.new_column("ALPHA", 0.0);
    t.new_column("BETA", 0.0);
    let (idx, col) = t.column_by_name("BETA").unwrap();
    assert_eq!(idx, 1);
    assert_eq!(col.name(), Some("BETA"));
    assert!(t.column_by_name("GAMMA").is_none());
}

#[test]
fn table_new_line_allocs_cells() {
    let mut t = Table::new();
    t.new_column("A", 0.0);
    t.new_column("B", 0.0);
    let id = t.new_line(None);
    assert_eq!(t.line(id).ncells(), 2);
}

#[test]
fn table_remove_all_columns() {
    let mut t = Table::new();
    t.new_column("A", 0.0);
    t.new_column("B", 0.0);
    t.remove_all_columns();
    assert_eq!(t.ncols(), 0);
}

#[test]
fn table_is_empty() {
    let mut t = Table::new();
    assert!(t.is_empty());
    t.new_column("A", 0.0);
    t.new_line(None);
    assert!(!t.is_empty());
}

// ── Table tree operations ─────────────────────────────────────────────────────

#[test]
fn table_tree_parent_child() {
    let mut t = Table::new();
    {
        let i = t.new_column("N", 0.0);
        t.column_mut(i).unwrap().set_tree(true);
    }

    let root = t.new_line(None);
    let child = t.new_line(Some(root));
    let grandchild = t.new_line(Some(child));

    assert_eq!(t.line(child).parent(), Some(root));
    assert_eq!(t.line(grandchild).parent(), Some(child));
    assert!(t.line(root).children().contains(&child));
    assert!(t.line(child).children().contains(&grandchild));
    assert_eq!(t.root_line_ids(), vec![root]);
}

#[test]
fn table_is_ancestor() {
    let mut t = Table::new();
    {
        let i = t.new_column("N", 0.0);
        t.column_mut(i).unwrap().set_tree(true);
    }

    let root = t.new_line(None);
    let child = t.new_line(Some(root));
    let grandchild = t.new_line(Some(child));

    assert!(t.is_ancestor(root, grandchild));
    assert!(t.is_ancestor(root, child));
    assert!(t.is_ancestor(child, grandchild));
    assert!(!t.is_ancestor(grandchild, root));
    assert!(!t.is_ancestor(child, root));
}

#[test]
fn table_add_child_reparents() {
    let mut t = Table::new();
    {
        let i = t.new_column("N", 0.0);
        t.column_mut(i).unwrap().set_tree(true);
    }

    let root1 = t.new_line(None);
    let root2 = t.new_line(None);
    let child = t.new_line(Some(root1));

    assert_eq!(t.line(child).parent(), Some(root1));
    t.add_child(root2, child);
    assert_eq!(t.line(child).parent(), Some(root2));
    assert!(!t.line(root1).children().contains(&child));
    assert!(t.line(root2).children().contains(&child));
}

#[test]
fn table_remove_child() {
    let mut t = Table::new();
    {
        let i = t.new_column("N", 0.0);
        t.column_mut(i).unwrap().set_tree(true);
    }

    let root = t.new_line(None);
    let child = t.new_line(Some(root));

    t.remove_child(root, child);
    assert!(t.line(root).children().is_empty());
    assert_eq!(t.line(child).parent(), None);
}

// ── Table sort ────────────────────────────────────────────────────────────────

#[test]
fn sort_children_alphabetically() {
    let mut t = Table::new();
    t.enable_no_headings(true);
    t.enable_ascii(true);
    {
        let i = t.new_column("NAME", 0.0);
        t.column_mut(i).unwrap().set_tree(true);
    }

    let root = t.new_line(None);
    t.line_mut(root).set_data(0, "root");

    for name in ["zebra", "apple", "mango"] {
        let row = t.new_line(Some(root));
        t.line_mut(row).set_data(0, name);
    }

    t.sort(0);

    let children = t.line(root).children().to_vec();
    assert_eq!(t.line(children[0]).get_data(0), Some("apple"));
    assert_eq!(t.line(children[1]).get_data(0), Some("mango"));
    assert_eq!(t.line(children[2]).get_data(0), Some("zebra"));
}

#[test]
fn sort_with_custom_cmp_func() {
    let mut t = Table::new();
    t.enable_no_headings(true);
    t.enable_ascii(true);
    {
        let i = t.new_column("VAL", 0.0);
        t.column_mut(i).unwrap().set_tree(true);
    }
    // Numeric comparator
    t.column_mut(0).unwrap().set_cmp_func(|a, b| {
        let av: i64 = a.data().unwrap_or("0").parse().unwrap_or(0);
        let bv: i64 = b.data().unwrap_or("0").parse().unwrap_or(0);
        av.cmp(&bv)
    });

    let root = t.new_line(None);
    t.line_mut(root).set_data(0, "root");

    for val in ["10", "2", "20"] {
        let row = t.new_line(Some(root));
        t.line_mut(row).set_data(0, val);
    }

    t.sort(0);

    let children = t.line(root).children().to_vec();
    assert_eq!(t.line(children[0]).get_data(0), Some("2"));
    assert_eq!(t.line(children[1]).get_data(0), Some("10"));
    assert_eq!(t.line(children[2]).get_data(0), Some("20"));
}

// ── Table clone ───────────────────────────────────────────────────────────────

#[test]
fn table_clone_is_independent() {
    let mut t = Table::new();
    t.new_column("X", 0.0);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "original");

    let mut clone = t.clone();
    clone.line_mut(row).set_data(0, "modified");

    assert_eq!(t.line(row).get_data(0), Some("original"));
    assert_eq!(clone.line(row).get_data(0), Some("modified"));
}

#[test]
fn table_clone_preserves_columns_and_lines() {
    let mut t = Table::new();
    {
        let i = t.new_column("A", 5.0);
        t.column_mut(i).unwrap().set_right(true);
    }
    t.new_column("B", 0.0);
    let r1 = t.new_line(None);
    t.line_mut(r1).set_data(0, "x").set_data(1, "y");

    let clone = t.clone();
    assert_eq!(clone.ncols(), 2);
    assert_eq!(clone.nlines(), 1);
    assert_eq!(clone.columns()[0].whint(), 5.0);
    assert!(clone.columns()[0].is_right());
    assert_eq!(clone.line(r1).get_data(0), Some("x"));
}

// ── Table configuration ───────────────────────────────────────────────────────

#[test]
fn table_name() {
    let mut t = Table::new();
    assert!(t.name().is_none());
    t.set_name("mytable");
    assert_eq!(t.name(), Some("mytable"));
}

#[test]
fn table_output_mode_predicates() {
    let mut t = Table::new();
    assert!(!t.is_json());
    assert!(!t.is_raw());
    assert!(!t.is_export());
    t.set_output_mode(OutputMode::Json);
    assert!(t.is_json());
    t.set_output_mode(OutputMode::Raw);
    assert!(t.is_raw());
    t.set_output_mode(OutputMode::Export);
    assert!(t.is_export());
}

#[test]
fn table_no_headings() {
    let mut t = Table::new();
    t.enable_no_headings(true);
    t.new_column("X", 0.0);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "data");
    let out = print_table_to_string(&t).unwrap();
    assert!(!out.contains("X"), "header should be absent");
    assert!(out.contains("data"));
}

#[test]
fn table_column_separator() {
    let mut t = Table::new();
    t.set_column_separator("|");
    t.enable_no_headings(true);
    t.new_column("A", 0.0);
    t.new_column("B", 0.0);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "x").set_data(1, "y");
    let out = print_table_to_string(&t).unwrap();
    assert!(out.contains("x|y"));
}

#[test]
fn table_termforce_always() {
    let mut t = Table::new();
    t.set_termforce(TermForce::Always);
    assert_eq!(t.termforce(), TermForce::Always);
}

#[test]
fn table_symbols_ascii() {
    let mut t = Table::new();
    t.enable_ascii(true);
    assert_eq!(t.symbols().branch, "|-");
    assert_eq!(t.symbols().right, "`-");
}

#[test]
fn table_custom_symbols() {
    let mut t = Table::new();
    let mut sym = Symbols::ascii();
    sym.branch = ">> ".to_string();
    t.set_symbols(sym);
    assert_eq!(t.symbols().branch, ">> ");
}

#[test]
fn table_minout_skips_empty_trailing() {
    let mut t = Table::new();
    t.enable_minout(true);
    t.enable_no_headings(true);
    t.new_column("A", 0.0);
    t.new_column("EMPTY", 0.0);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "hello"); // EMPTY column has no data
    let out = print_table_to_string(&t).unwrap();
    assert_eq!(out.trim(), "hello"); // trailing empty column omitted
}

// ── Output format correctness ─────────────────────────────────────────────────

#[test]
fn normal_right_align() {
    let mut t = Table::new();
    t.enable_no_headings(true);
    {
        let i = t.new_column("NUM", 5.0);
        t.column_mut(i).unwrap().set_right(true);
    }
    for v in ["1", "99", "1000"] {
        let row = t.new_line(None);
        t.line_mut(row).set_data(0, v);
    }
    let out = print_table_to_string(&t).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].ends_with("1"),
        "smallest should be right-padded: {:?}",
        lines[0]
    );
    assert!(lines[2].ends_with("1000"));
}

#[test]
fn raw_mode_no_padding() {
    let mut t = Table::new();
    t.set_output_mode(OutputMode::Raw);
    t.new_column("A", 20.0);
    t.new_column("B", 20.0);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "x").set_data(1, "y");
    let out = print_table_to_string(&t).unwrap();
    // Raw mode: no padding, just separator
    assert!(out.lines().any(|l| l == "x y"), "raw data line: {:?}", out);
}

#[test]
fn export_mode_escaping() {
    let mut t = Table::new();
    t.set_output_mode(OutputMode::Export);
    t.new_column("PATH", 0.0);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, r#"say "hello""#);
    let out = print_table_to_string(&t).unwrap();
    assert!(out.contains(r#"PATH="say \"hello\"""#));
}

#[test]
fn json_number_type() {
    let mut t = Table::new();
    t.set_name("items");
    t.set_output_mode(OutputMode::Json);
    let mut col = Column::with_name("COUNT", 0.0);
    col.set_json_type(JsonType::Number);
    t.add_column(col);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "42");
    let out = print_table_to_string(&t).unwrap();
    assert!(out.contains("\"COUNT\": 42"), "got: {out}");
}

#[test]
fn json_boolean_type() {
    let mut t = Table::new();
    t.set_name("items");
    t.set_output_mode(OutputMode::Json);
    let mut col = Column::with_name("ENABLED", 0.0);
    col.set_json_type(JsonType::Boolean);
    t.add_column(col);
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "1");
    let out = print_table_to_string(&t).unwrap();
    assert!(out.contains("\"ENABLED\": true"), "got: {out}");
}

#[test]
fn hidden_column_excluded_from_output() {
    let mut t = Table::new();
    t.enable_no_headings(true);
    t.new_column("VISIBLE", 0.0);
    {
        let i = t.new_column("SECRET", 0.0);
        t.column_mut(i).unwrap().set_hidden(true);
    }
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "shown").set_data(1, "hidden");
    let out = print_table_to_string(&t).unwrap();
    assert!(out.contains("shown"));
    assert!(!out.contains("hidden"));
}

#[test]
fn wrap_flag_splits_long_data() {
    let mut t = Table::new();
    t.set_termforce(TermForce::Always);
    // Narrow terminal: ID=4, sep=1, TEXT=8 → total=13, but available=12
    // so TEXT gets squeezed to 7. Data "hello world" (11 chars) > 7 → wraps.
    t.set_termwidth(13);
    t.enable_no_headings(true);
    t.new_column("ID", 4.0);
    {
        let i = t.new_column("TEXT", 8.0);
        t.column_mut(i).unwrap().set_wrap(true);
    }
    let row = t.new_line(None);
    t.line_mut(row).set_data(0, "1").set_data(1, "hello world");
    let out = print_table_to_string(&t).unwrap();
    // With wrapping, the text splits across multiple output lines.
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines.len() >= 2,
        "wrap should produce multiple lines: {out:?}"
    );
    // All pieces of the wrapped text should be present.
    let combined: String =
        lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join("");
    assert!(combined.contains("hello"));
    assert!(combined.contains("world"));
}

// ── Filter ────────────────────────────────────────────────────────────────────

#[test]
fn filter_string_equality() {
    let f = Filter::parse(r#"TYPE == "disk""#).unwrap();
    assert!(
        f.evaluate(|c| if c == "TYPE" {
            Some("disk".into())
        } else {
            None
        })
        .unwrap()
    );
    assert!(
        !f.evaluate(|c| if c == "TYPE" {
            Some("part".into())
        } else {
            None
        })
        .unwrap()
    );
}

#[test]
fn filter_integer_suffix() {
    let f = Filter::parse("SIZE > 1G").unwrap();
    let big = f
        .evaluate(|c| {
            if c == "SIZE" {
                Some("2147483648".into())
            } else {
                None
            }
        })
        .unwrap();
    assert!(big);
    let small = f
        .evaluate(|c| {
            if c == "SIZE" {
                Some("100".into())
            } else {
                None
            }
        })
        .unwrap();
    assert!(!small);
}

#[test]
fn filter_boolean_and_or() {
    let f = Filter::parse(r#"TYPE == "disk" && SIZE > 0"#).unwrap();
    assert!(
        f.evaluate(|c| match c {
            "TYPE" => Some("disk".into()),
            "SIZE" => Some("1024".into()),
            _ => None,
        })
        .unwrap()
    );
    assert!(
        !f.evaluate(|c| match c {
            "TYPE" => Some("part".into()),
            "SIZE" => Some("1024".into()),
            _ => None,
        })
        .unwrap()
    );
}

#[test]
fn filter_parse_error_on_invalid_input() {
    assert!(Filter::parse("SIZE &&& NAME").is_err());
}

#[test]
fn filter_clone() {
    let f = Filter::parse("SIZE > 0").unwrap();
    let clone = f.clone();
    let result = clone
        .evaluate(|c| if c == "SIZE" { Some("1".into()) } else { None })
        .unwrap();
    assert!(result);
}

// ── Group ─────────────────────────────────────────────────────────────────────

#[test]
fn group_member_and_child() {
    let mut t = Table::new();
    t.new_column("N", 0.0);

    let g = t.new_group();
    let m1 = t.new_line(None);
    let m2 = t.new_line(None);
    let child = t.new_line(None);

    t.group_add_member(g, m1);
    t.group_add_member(g, m2);
    t.group_add_child(g, child);

    assert_eq!(t.line(m1).parent(), None); // group members aren't tree children
    assert_eq!(t.nlines(), 3);
}
