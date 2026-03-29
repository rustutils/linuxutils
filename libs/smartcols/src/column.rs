//! Column — defines formatting, type, and behavior for one vertical slice
//! of the table.
//!
//! Each column has a name (shown in headers), a width hint that guides the
//! print engine's layout algorithm, behavioral flags, and optional metadata
//! for JSON/filter output.

use crate::cell::Cell;
use bitflags::bitflags;
use std::{cmp::Ordering, fmt, sync::Arc};

bitflags! {
    /// Internal bitfield matching the C `SCOLS_FL_*` constants.
    ///
    /// Not part of the public API — use the individual setter methods on
    /// [`Column`] instead (e.g. [`Column::set_tree`], [`Column::set_right`]).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub(crate) struct ColumnFlags: u32 {
        const TRUNC        = 1 << 0;
        const TREE         = 1 << 1;
        const RIGHT        = 1 << 2;
        const STRICT_WIDTH = 1 << 3;
        const NO_EXTREMES  = 1 << 4;
        const HIDDEN       = 1 << 5;
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
#[derive(Clone)]
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

type CmpFunc = Arc<dyn Fn(&Cell, &Cell) -> Ordering + Send + Sync>;

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

    /// Create a column with a name and width hint.
    ///
    /// The `whint` value guides the print engine's layout:
    /// - `>= 1.0` — absolute character width (e.g., `6.0` = 6 chars)
    /// - `0.0 < whint < 1.0` — fraction of terminal width (e.g., `0.25` = 25%)
    /// - `0.0` — no hint, size to content
    ///
    /// Use the individual setter methods to configure flags:
    /// ```
    /// use smartcols::Column;
    /// let mut col = Column::with_name("NAME", 10.0);
    /// col.set_tree(true).set_no_extremes(true);
    /// ```
    pub fn with_name(name: &str, whint: f64) -> Self {
        let mut col = Self::new();
        col.set_name(name);
        col.whint = whint;
        col
    }

    /// Create a column with a name, width hint, and flags bitmask (for FFI).
    pub(crate) fn with_name_and_flags(
        name: &str,
        whint: f64,
        flags: ColumnFlags,
    ) -> Self {
        let mut col = Self::with_name(name, whint);
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

    pub(crate) fn set_flags(&mut self, flags: ColumnFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    pub(crate) fn flags(&self) -> ColumnFlags {
        self.flags
    }

    /// Mark this column as the tree hierarchy column.
    pub fn set_tree(&mut self, enable: bool) -> &mut Self {
        self.flags.set(ColumnFlags::TREE, enable);
        self
    }

    pub fn is_tree(&self) -> bool {
        self.flags.contains(ColumnFlags::TREE)
    }

    /// Truncate cell data that exceeds the column width.
    pub fn set_trunc(&mut self, enable: bool) -> &mut Self {
        self.flags.set(ColumnFlags::TRUNC, enable);
        self
    }

    pub fn is_trunc(&self) -> bool {
        self.flags.contains(ColumnFlags::TRUNC)
    }

    /// Right-align cell data in this column.
    pub fn set_right(&mut self, enable: bool) -> &mut Self {
        self.flags.set(ColumnFlags::RIGHT, enable);
        self
    }

    pub fn is_right(&self) -> bool {
        self.flags.contains(ColumnFlags::RIGHT)
    }

    /// Never shrink this column below its hint width.
    pub fn set_strict_width(&mut self, enable: bool) -> &mut Self {
        self.flags.set(ColumnFlags::STRICT_WIDTH, enable);
        self
    }

    pub fn is_strict_width(&self) -> bool {
        self.flags.contains(ColumnFlags::STRICT_WIDTH)
    }

    /// Ignore outlier cell values when calculating column width.
    pub fn set_no_extremes(&mut self, enable: bool) -> &mut Self {
        self.flags.set(ColumnFlags::NO_EXTREMES, enable);
        self
    }

    pub fn is_no_extremes(&self) -> bool {
        self.flags.contains(ColumnFlags::NO_EXTREMES)
    }

    /// Store data in memory but omit from printed output.
    pub fn set_hidden(&mut self, enable: bool) -> &mut Self {
        self.flags.set(ColumnFlags::HIDDEN, enable);
        self
    }

    pub fn is_hidden(&self) -> bool {
        self.flags.contains(ColumnFlags::HIDDEN)
    }

    /// Wrap long cell data across multiple output lines.
    pub fn set_wrap(&mut self, enable: bool) -> &mut Self {
        self.flags.set(ColumnFlags::WRAP, enable);
        self
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
        F: Fn(&Cell, &Cell) -> Ordering + Send + Sync + 'static,
    {
        self.cmp_func = Some(Arc::new(f));
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
