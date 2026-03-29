//! Line — a row in the table.
//!
//! Each line holds one [`Cell`] per column and can participate
//! in tree hierarchies (parent/child) and group relationships.
//! Lines are created via [`Table::new_line`](crate::Table::new_line) and
//! accessed via [`LineId`] handles.

use crate::cell::Cell;

/// Opaque handle to a line within a [`Table`](crate::Table).
///
/// Obtained from [`Table::new_line`](crate::Table::new_line) and used to
/// reference lines in tree and data operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(pub(crate) usize);

/// A row in the table — holds one cell per column, plus optional tree
/// parent/child relationships.
#[derive(Debug, Clone, Default)]
pub struct Line {
    cells: Vec<Cell>,
    color: Option<String>,
    pub(crate) parent: Option<LineId>,
    pub(crate) children: Vec<LineId>,
    pub(crate) group_member: Option<GroupId>,
    pub(crate) group_child: Option<GroupId>,
}

/// Opaque handle to a group within a [`Table`](crate::Table).
///
/// Obtained from [`Table::new_group`](crate::Table::new_group).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupId(pub(crate) usize);

impl Line {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensure this line has at least `n` cells.
    pub fn alloc_cells(&mut self, n: usize) {
        if self.cells.len() < n {
            self.cells.resize_with(n, Cell::new);
        }
    }

    pub fn cell(&self, idx: usize) -> Option<&Cell> {
        self.cells.get(idx)
    }

    pub fn cell_mut(&mut self, idx: usize) -> Option<&mut Cell> {
        self.cells.get_mut(idx)
    }

    pub fn ncells(&self) -> usize {
        self.cells.len()
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set data for cell at column index `n`, allocating if needed.
    pub fn set_data(&mut self, n: usize, data: &str) -> &mut Self {
        if n >= self.cells.len() {
            self.alloc_cells(n + 1);
        }
        self.cells[n].set_data(data);
        self
    }

    /// Get data for cell at column index `n`.
    pub fn get_data(&self, n: usize) -> Option<&str> {
        self.cells.get(n).and_then(|c| c.data())
    }

    /// Returns true if cell at index `n` has data.
    pub fn is_filled(&self, n: usize) -> bool {
        self.cells.get(n).is_some_and(|c| c.is_filled())
    }

    pub fn set_color(&mut self, color: &str) -> &mut Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn parent(&self) -> Option<LineId> {
        self.parent
    }

    pub fn children(&self) -> &[LineId] {
        &self.children
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}
