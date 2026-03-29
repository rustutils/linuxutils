//! Column — defines formatting, type, and behavior for one vertical slice
//! of the table.
//!
//! Each column has a name (shown in headers), a width hint that guides the
//! print engine's layout algorithm, behavioral flags, and optional metadata
//! for JSON/filter output.

use crate::cell::Cell;
use bitflags::bitflags;
use std::{cmp::Ordering, fmt};

bitflags! {
    /// Behavioral flags for a column.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ColumnFlags: u32 {
        /// Truncate cell data if it exceeds the column width.
        const TRUNC        = 1 << 0;
        /// This column renders the tree hierarchy.
        const TREE         = 1 << 1;
        /// Right-align cell data.
        const RIGHT        = 1 << 2;
        /// Don't reduce width even if the column is empty.
        const STRICT_WIDTH = 1 << 3;
        /// Ignore extreme outlier values when calculating width.
        const NO_EXTREMES  = 1 << 4;
        /// Keep data in memory but don't print the column.
        const HIDDEN       = 1 << 5;
        /// Wrap long cell data across multiple lines.
        const WRAP         = 1 << 6;
    }
}

/// Controls how a column's data is serialized in JSON output mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsonType {
    #[default]
    String,
    Number,
    Boolean,
    ArrayString,
    ArrayNumber,
    BooleanOptional,
    Float,
}

/// Data type used when evaluating filter expressions and counters against
/// column values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataType {
    #[default]
    None,
    U64,
    Boolean,
    Float,
    String,
}

/// A column definition — describes formatting, type, and behavior for one
/// vertical slice of the table.
pub struct Column {
    name: Option<String>,
    header: Cell,
    whint: f64,
    flags: ColumnFlags,
    json_type: JsonType,
    data_type: DataType,
    color: Option<String>,
    header_color: Option<String>,
    safechars: Option<String>,
    uri: Option<String>,
    annotation: Option<String>,
    cmp_func: Option<CmpFunc>,
    /// Calculated at print time.
    width: Option<usize>,
}

type CmpFunc = Box<dyn Fn(&Cell, &Cell) -> Ordering>;

impl Column {
    /// Create a column with default settings (no name, zero width hint, no flags).
    pub fn new() -> Self {
        Self {
            name: None,
            header: Cell::new(),
            whint: 0.0,
            flags: ColumnFlags::empty(),
            json_type: JsonType::default(),
            data_type: DataType::default(),
            color: None,
            header_color: None,
            safechars: None,
            uri: None,
            annotation: None,
            cmp_func: None,
            width: None,
        }
    }

    /// Create a column with a name, width hint, and flags.
    ///
    /// The `whint` value guides the print engine's layout:
    /// - `>= 1.0` — absolute character width (e.g., `6.0` = 6 chars)
    /// - `0.0 < whint < 1.0` — fraction of terminal width (e.g., `0.25` = 25%)
    /// - `0.0` — no hint, size to content
    pub fn with_name(name: &str, whint: f64, flags: ColumnFlags) -> Self {
        let mut col = Self::new();
        col.set_name(name);
        col.whint = whint;
        col.flags = flags;
        col
    }

    /// Set the column name. Also sets the header cell text.
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self.header.set_data(name);
        self
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Column name sanitized for use as a shell variable (non-alnum replaced
    /// with `_`).
    pub fn name_as_shellvar(&self) -> Option<String> {
        self.name.as_ref().map(|n| to_shellvar(n))
    }

    pub fn header(&self) -> &Cell {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Cell {
        &mut self.header
    }

    pub fn set_whint(&mut self, whint: f64) -> &mut Self {
        self.whint = whint;
        self
    }

    pub fn whint(&self) -> f64 {
        self.whint
    }

    /// The calculated width, only available after printing has begun.
    pub fn width(&self) -> Option<usize> {
        self.width
    }

    #[expect(dead_code, reason = "will be used by print engine")]
    pub(crate) fn set_width(&mut self, w: usize) {
        self.width = Some(w);
    }

    pub fn set_flags(&mut self, flags: ColumnFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    pub fn flags(&self) -> ColumnFlags {
        self.flags
    }

    pub fn is_tree(&self) -> bool {
        self.flags.contains(ColumnFlags::TREE)
    }

    pub fn is_trunc(&self) -> bool {
        self.flags.contains(ColumnFlags::TRUNC)
    }

    pub fn is_right(&self) -> bool {
        self.flags.contains(ColumnFlags::RIGHT)
    }

    pub fn is_strict_width(&self) -> bool {
        self.flags.contains(ColumnFlags::STRICT_WIDTH)
    }

    pub fn is_no_extremes(&self) -> bool {
        self.flags.contains(ColumnFlags::NO_EXTREMES)
    }

    pub fn is_hidden(&self) -> bool {
        self.flags.contains(ColumnFlags::HIDDEN)
    }

    pub fn is_wrap(&self) -> bool {
        self.flags.contains(ColumnFlags::WRAP)
    }

    pub fn set_json_type(&mut self, jt: JsonType) -> &mut Self {
        self.json_type = jt;
        self
    }

    pub fn json_type(&self) -> JsonType {
        self.json_type
    }

    pub fn set_data_type(&mut self, dt: DataType) -> &mut Self {
        self.data_type = dt;
        self
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    pub fn set_color(&mut self, color: &str) -> &mut Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn set_header_color(&mut self, color: &str) -> &mut Self {
        self.header_color = Some(color.to_string());
        self
    }

    pub fn header_color(&self) -> Option<&str> {
        self.header_color.as_deref()
    }

    pub fn set_safechars(&mut self, chars: &str) -> &mut Self {
        self.safechars = Some(chars.to_string());
        self
    }

    pub fn safechars(&self) -> Option<&str> {
        self.safechars.as_deref()
    }

    pub fn set_uri(&mut self, uri: &str) -> &mut Self {
        self.uri = Some(uri.to_string());
        self
    }

    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    pub fn set_annotation(&mut self, ann: &str) -> &mut Self {
        self.annotation = Some(ann.to_string());
        self
    }

    pub fn annotation(&self) -> Option<&str> {
        self.annotation.as_deref()
    }

    pub fn set_cmp_func<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&Cell, &Cell) -> Ordering + 'static,
    {
        self.cmp_func = Some(Box::new(f));
        self
    }

    pub fn cmp_cells(&self, a: &Cell, b: &Cell) -> Ordering {
        if let Some(f) = &self.cmp_func {
            f(a, b)
        } else {
            crate::cell::cmp_cells_str(a, b)
        }
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Column")
            .field("name", &self.name)
            .field("whint", &self.whint)
            .field("flags", &self.flags)
            .field("json_type", &self.json_type)
            .field("data_type", &self.data_type)
            .field("has_cmp_func", &self.cmp_func.is_some())
            .finish()
    }
}

/// Convert a name to a valid shell variable name: non-alphanumeric chars
/// become `_`.
pub fn to_shellvar(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
