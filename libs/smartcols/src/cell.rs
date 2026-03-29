//! Cell — the smallest unit in a table.
//!
//! Each cell holds an optional data string plus formatting metadata (color,
//! URI, alignment). Cells are stored inside [`Line`](crate::Line)s,
//! one per column.

/// The smallest unit in a table — holds a data string plus optional
/// color, URI, and alignment.
#[derive(Debug, Clone, Default)]
pub struct Cell {
    data: Option<String>,
    color: Option<String>,
    uri: Option<String>,
    alignment: Alignment,
}

impl Cell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_data(&mut self, data: &str) -> &mut Self {
        self.data = Some(data.to_string());
        self
    }

    pub fn data(&self) -> Option<&str> {
        self.data.as_deref()
    }

    pub fn clear_data(&mut self) -> &mut Self {
        self.data = None;
        self
    }

    pub fn set_color(&mut self, color: &str) -> &mut Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn set_uri(&mut self, uri: &str) -> &mut Self {
        self.uri = Some(uri.to_string());
        self
    }

    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    pub fn set_alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.alignment = alignment;
        self
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    /// Set alignment from a C `SCOLS_CELL_FL_*` bitmask (for FFI).
    pub(crate) fn set_flags_from_bits(&mut self, bits: u32) {
        self.alignment = if bits & (1 << 1) != 0 {
            Alignment::Right
        } else if bits & (1 << 0) != 0 {
            Alignment::Center
        } else {
            Alignment::Left
        };
    }

    pub fn is_filled(&self) -> bool {
        self.data.is_some()
    }

    /// Reset the cell to its default (empty) state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Text alignment within a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Compare two cells by their data strings.
pub fn cmp_cells_str(a: &Cell, b: &Cell) -> std::cmp::Ordering {
    let a = a.data().unwrap_or("");
    let b = b.data().unwrap_or("");
    a.cmp(b)
}
