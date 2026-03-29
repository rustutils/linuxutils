//! Smart adaptive formatting of tabular data.
//!
//! This crate provides an idiomatic Rust library for building and printing
//! tables with automatic column width calculation, tree rendering, multiple
//! output formats, and a filter expression language. It also exposes an
//! ABI-compatible C FFI matching the original `libsmartcols` from util-linux.
//!
//! # Quick start
//!
//! ```
//! use smartcols::{Table, Column, print_table_to_string};
//!
//! let mut table = Table::new();
//! table.new_column("NAME", 10.0);
//! let mut size_col = Column::with_name("SIZE", 6.0);
//! size_col.set_right(true);
//! table.add_column(size_col);
//!
//! let l1 = table.new_line(None);
//! table.line_mut(l1).set_data(0, "sda").set_data(1, "100G");
//!
//! let l2 = table.new_line(None);
//! table.line_mut(l2).set_data(0, "sdb").set_data(1, "50G");
//!
//! let output = print_table_to_string(&table).unwrap();
//! assert!(output.contains("sda"));
//! assert!(output.contains("100G"));
//! ```
//!
//! # Output modes
//!
//! Set via [`Table::set_output_mode`]:
//!
//! ```
//! use smartcols::{OutputMode, Table, print_table_to_string};
//!
//! let mut table = Table::new();
//! table.set_name("devices");
//! table.new_column("NAME", 0.0);
//! table.new_column("SIZE", 0.0);
//!
//! let row = table.new_line(None);
//! table.line_mut(row).set_data(0, "sda").set_data(1, "100G");
//!
//! // JSON output
//! table.set_output_mode(OutputMode::Json);
//! let json = print_table_to_string(&table).unwrap();
//! assert!(json.contains("\"NAME\": \"sda\""));
//!
//! // Export (shell-friendly KEY="value" pairs)
//! table.set_output_mode(OutputMode::Export);
//! let export = print_table_to_string(&table).unwrap();
//! assert!(export.contains("NAME=\"sda\""));
//! ```
//!
//! | Mode | Description |
//! |------|-------------|
//! | [`OutputMode::Normal`] | Human-readable with column alignment and padding |
//! | [`OutputMode::Raw`] | Compact, separator-delimited (no padding) |
//! | [`OutputMode::Export`] | `NAME="value"` pairs for shell consumption |
//! | [`OutputMode::Json`] | JSON array of objects |
//! | [`OutputMode::ShellVar`] | Like Export but with shell-safe column names |
//!
//! # Column width hints
//!
//! The `whint` parameter passed to [`Table::new_column`] guides the print
//! engine's layout algorithm:
//!
//! - **`>= 1.0`** — absolute minimum character width (e.g. `10.0` = at least
//!   10 chars)
//! - **`0.0 < whint < 1.0`** — fraction of terminal width (e.g. `0.25` = 25%);
//!   only applied when output is a terminal
//! - **`0.0`** — no hint; column is sized to fit its content
//!
//! # Column flags
//!
//! Per-column rendering behavior is controlled via setter methods on [`Column`]:
//!
//! | Method | Effect |
//! |--------|--------|
//! | [`Column::set_tree`] | Draws tree connectors (use with [`Table::new_line`] parent) |
//! | [`Column::set_right`] | Right-aligns cell data |
//! | [`Column::set_trunc`] | Truncates data that exceeds the column width |
//! | [`Column::set_wrap`] | Wraps long data across multiple output lines |
//! | [`Column::set_hidden`] | Stores data in memory but omits it from output |
//! | [`Column::set_strict_width`] | Never shrink the column below its hint width |
//! | [`Column::set_no_extremes`] | Ignore outlier values when calculating widths |
//!
//! # Tree rendering
//!
//! Mark a column as a tree column with [`Column::set_tree`] and pass a parent
//! [`LineId`] when creating child lines. The print engine draws Unicode (or
//! ASCII) tree connectors automatically.
//!
//! ```
//! use smartcols::{Table, print_table_to_string};
//!
//! let mut table = Table::new();
//! table.enable_no_headings(true);
//! table.enable_ascii(true);
//! let idx = table.new_column("NAME", 0.0);
//! table.column_mut(idx).unwrap().set_tree(true);
//!
//! let root = table.new_line(None);
//! table.line_mut(root).set_data(0, "parent");
//!
//! let child1 = table.new_line(Some(root));
//! table.line_mut(child1).set_data(0, "child-a");
//!
//! let child2 = table.new_line(Some(root));
//! table.line_mut(child2).set_data(0, "child-b");
//!
//! let output = print_table_to_string(&table).unwrap();
//! assert!(output.contains("|-child-a"));
//! assert!(output.contains("`-child-b"));
//! ```
//!
//! # Sorting
//!
//! Sort child lines under a parent using [`Table::sort`]. A custom comparator
//! can be set per-column with [`Column::set_cmp_func`]. Sorting reorders
//! children within each parent node; for flat tables, add rows as children of
//! a single root line.
//!
//! ```
//! use smartcols::{Table, print_table_to_string};
//!
//! let mut table = Table::new();
//! table.enable_ascii(true);
//! table.enable_no_headings(true);
//! let idx = table.new_column("NAME", 0.0);
//! table.column_mut(idx).unwrap().set_tree(true);
//!
//! let root = table.new_line(None);
//! table.line_mut(root).set_data(0, "root");
//!
//! for name in ["zebra", "apple", "mango"] {
//!     let row = table.new_line(Some(root));
//!     table.line_mut(row).set_data(0, name);
//! }
//!
//! table.sort(0); // sort children of root by column 0
//!
//! let output = print_table_to_string(&table).unwrap();
//! let lines: Vec<&str> = output.lines().collect();
//! // lines[0] = root, children appear sorted after it
//! assert!(lines[1].contains("apple"));
//! assert!(lines[2].contains("mango"));
//! assert!(lines[3].contains("zebra"));
//! ```
//!
//! # Groups
//!
//! Groups express M:N relationships between lines — multiple lines can be
//! "members" of a group, and other lines can be "children" of that group.
//! The print engine draws bracket connectors on the left margin.
//!
//! ```
//! use smartcols::{Table, TermForce, print_table_to_string};
//!
//! let mut table = Table::new();
//! table.enable_no_headings(true);
//! table.enable_ascii(true);
//! table.set_termwidth(40);
//! table.set_termforce(TermForce::Always);
//! table.new_column("NAME", 0.0);
//!
//! let group = table.new_group();
//!
//! let m1 = table.new_line(None);
//! table.line_mut(m1).set_data(0, "server");
//! let m2 = table.new_line(None);
//! table.line_mut(m2).set_data(0, "worker");
//!
//! let child = table.new_line(None);
//! table.line_mut(child).set_data(0, "logger");
//!
//! table.group_add_member(group, m1);
//! table.group_add_member(group, m2);
//! table.group_add_child(group, child);
//!
//! let output = print_table_to_string(&table).unwrap();
//! assert!(output.contains(",-server"));
//! assert!(output.contains("|-worker"));
//! assert!(output.contains("`-logger"));
//! ```
//!
//! # Cloning
//!
//! Both [`Table`] and [`Column`] implement [`Clone`]. Cloning a table produces
//! an independent copy including all lines and column definitions. If a column
//! has a custom comparator set via [`Column::set_cmp_func`], the clone shares
//! the same comparator function (via [`Arc`](std::sync::Arc)).
//!
//! ```
//! use smartcols::Table;
//!
//! let mut table = Table::new();
//! table.new_column("X", 0.0);
//! let row = table.new_line(None);
//! table.line_mut(row).set_data(0, "hello");
//!
//! let clone = table.clone();
//! assert_eq!(clone.nlines(), table.nlines());
//! ```
//!
//! # Filtering
//!
//! Parse filter expressions with [`Filter::parse`] and evaluate them against
//! line data. Supports comparisons, boolean logic, regex matching, and
//! integer suffixes (K, M, G, ...).
//!
//! ```
//! use smartcols::Filter;
//!
//! let f = Filter::parse(r#"SIZE > 1G && TYPE == "disk""#).unwrap();
//! let matches = f.evaluate(|col| match col {
//!     "SIZE" => Some("4294967296".into()), // 4G
//!     "TYPE" => Some("disk".into()),
//!     _ => None,
//! }).unwrap();
//! assert!(matches);
//! ```

pub mod cell;
pub mod column;
pub mod ffi;
pub mod filter;
pub mod line;
pub mod print;
pub mod symbols;
pub mod table;

pub use cell::{Alignment, Cell};
pub use column::{Column, DataType, JsonType};
pub use filter::Filter;
pub use line::{GroupId, Line, LineId};
pub use print::{print_table, print_table_to_string};
pub use symbols::Symbols;
pub use table::{OutputMode, Table, TermForce};
