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
//! use smartcols::{Table, ColumnFlags, print_table_to_string};
//!
//! let mut table = Table::new();
//! table.new_column("NAME", 10.0, ColumnFlags::empty());
//! table.new_column("SIZE", 6.0, ColumnFlags::RIGHT);
//!
//! let l1 = table.new_line(None);
//! table.line_mut(l1).set_data(0, "sda").set_data(1, "100G");
//!
//! let l2 = table.new_line(None);
//! table.line_mut(l2).set_data(0, "sdb").set_data(1, "50G");
//!
//! let output = print_table_to_string(&table).unwrap();
//! assert!(output.contains("sda"));
//! ```
//!
//! # Output modes
//!
//! Set via [`Table::set_output_mode`]:
//! - **Normal** — human-readable with padding and alignment
//! - **Raw** — compact, machine-parseable
//! - **Export** — `NAME="value"` shell-friendly format
//! - **Json** — JSON array of objects
//! - **ShellVar** — like export with shell-safe column names
//!
//! # Tree rendering
//!
//! Mark a column with [`ColumnFlags::TREE`] and use parent arguments in
//! [`Table::new_line`] to build hierarchies. The print engine draws tree
//! connectors automatically.
//!
//! # Filtering
//!
//! Parse filter expressions with [`Filter::parse`] and evaluate them against
//! line data. Supports comparisons, boolean logic, regex matching, and
//! integer suffixes (K, M, G, ...).

pub mod cell;
pub mod column;
pub mod ffi;
pub mod filter;
pub mod line;
pub mod print;
pub mod symbols;
pub mod table;

pub use cell::{Alignment, Cell, CellFlags};
pub use column::{Column, ColumnFlags, DataType, JsonType};
pub use filter::Filter;
pub use line::{GroupId, Line, LineId};
pub use print::{print_table, print_table_to_string};
pub use symbols::Symbols;
pub use table::{OutputMode, Table, TermForce};
