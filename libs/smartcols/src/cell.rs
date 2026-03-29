use bitflags::bitflags;

bitflags! {
    /// Alignment flags for a cell.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CellFlags: u32 {
        const CENTER = 1 << 0;
        const RIGHT  = 1 << 1;
    }
}

/// The smallest unit in a table — holds a data string plus optional formatting.
#[derive(Debug, Clone, Default)]
pub struct Cell {
    data: Option<String>,
    color: Option<String>,
    uri: Option<String>,
    flags: CellFlags,
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

    pub fn set_flags(&mut self, flags: CellFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    pub fn flags(&self) -> CellFlags {
        self.flags
    }

    /// Returns the effective alignment: Right > Center > Left (default).
    pub fn alignment(&self) -> Alignment {
        if self.flags.contains(CellFlags::RIGHT) {
            Alignment::Right
        } else if self.flags.contains(CellFlags::CENTER) {
            Alignment::Center
        } else {
            Alignment::Left
        }
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
