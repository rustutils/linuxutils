use crate::{
    cell::Cell,
    column::{Column, ColumnFlags},
    line::{GroupId, Line, LineId},
    symbols::Symbols,
};

/// Output format mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    /// Human-readable columnar output with padding and alignment.
    #[default]
    Normal,
    /// Compact parsable format.
    Raw,
    /// `NAME="value"` format for machine parsing.
    Export,
    /// JSON array of objects.
    Json,
    /// Shell variable assignment format.
    ShellVar,
}

/// Controls whether terminal-aware formatting (tree symbols, width
/// detection) is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TermForce {
    #[default]
    Auto,
    Never,
    Always,
}

/// A group — tracks members and children for M:N line relationships.
#[derive(Debug, Clone, Default)]
pub(crate) struct Group {
    pub members: Vec<LineId>,
    pub children: Vec<LineId>,
}

/// The top-level table object. Owns all columns, lines, and configuration.
#[derive(Debug)]
pub struct Table {
    name: Option<String>,
    columns: Vec<Column>,
    lines: Vec<Line>,
    groups: Vec<Group>,
    symbols: Symbols,
    title: Cell,

    // Output configuration
    output_mode: OutputMode,
    colors_enabled: bool,
    no_headings: bool,
    header_repeat: bool,
    maxout: bool,
    minout: bool,
    no_wrap: bool,
    no_linesep: bool,
    no_encoding: bool,
    ascii: bool,
    column_separator: String,
    line_separator: String,

    // Terminal
    termforce: TermForce,
    termwidth: Option<usize>,
    termheight: Option<usize>,
    termwidth_reduce: usize,

    // Sorting
    sort_column: Option<usize>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            name: None,
            columns: Vec::new(),
            lines: Vec::new(),
            groups: Vec::new(),
            symbols: Symbols::default(),
            title: Cell::new(),

            output_mode: OutputMode::default(),
            colors_enabled: false,
            no_headings: false,
            header_repeat: false,
            maxout: false,
            minout: false,
            no_wrap: false,
            no_linesep: false,
            no_encoding: false,
            ascii: false,
            column_separator: "  ".to_string(),
            line_separator: "\n".to_string(),

            termforce: TermForce::default(),
            termwidth: None,
            termheight: None,
            termwidth_reduce: 0,

            sort_column: None,
        }
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn title(&self) -> &Cell {
        &self.title
    }

    pub fn title_mut(&mut self) -> &mut Cell {
        &mut self.title
    }

    /// Add a pre-built column and return its index.
    pub fn add_column(&mut self, col: Column) -> usize {
        let idx = self.columns.len();
        self.columns.push(col);
        // Ensure all existing lines have enough cells.
        for line in &mut self.lines {
            line.alloc_cells(self.columns.len());
        }
        idx
    }

    /// Create a new column with the given name, width hint, and flags.
    /// Returns the column index.
    pub fn new_column(
        &mut self,
        name: &str,
        whint: f64,
        flags: ColumnFlags,
    ) -> usize {
        self.add_column(Column::with_name(name, whint, flags))
    }

    pub fn column(&self, idx: usize) -> Option<&Column> {
        self.columns.get(idx)
    }

    pub fn column_mut(&mut self, idx: usize) -> Option<&mut Column> {
        self.columns.get_mut(idx)
    }

    pub fn column_by_name(&self, name: &str) -> Option<(usize, &Column)> {
        self.columns
            .iter()
            .enumerate()
            .find(|(_, c)| c.name() == Some(name))
    }

    pub fn ncols(&self) -> usize {
        self.columns.len()
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn remove_column(&mut self, idx: usize) -> Column {
        self.columns.remove(idx)
    }

    pub fn remove_all_columns(&mut self) {
        self.columns.clear();
    }

    /// Add a new line, optionally as a child of `parent`. Returns its `LineId`.
    pub fn new_line(&mut self, parent: Option<LineId>) -> LineId {
        let id = LineId(self.lines.len());
        let mut line = Line::new();
        line.alloc_cells(self.columns.len());
        line.parent = parent;
        self.lines.push(line);

        if let Some(pid) = parent {
            self.lines[pid.0].children.push(id);
        }
        id
    }

    pub fn line(&self, id: LineId) -> &Line {
        &self.lines[id.0]
    }

    pub fn line_mut(&mut self, id: LineId) -> &mut Line {
        &mut self.lines[id.0]
    }

    pub fn nlines(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Iterate over all line IDs.
    pub fn line_ids(&self) -> impl Iterator<Item = LineId> {
        (0..self.lines.len()).map(LineId)
    }

    /// Iterate over root (parentless) line IDs.
    pub fn root_line_ids(&self) -> Vec<LineId> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.parent.is_none())
            .map(|(i, _)| LineId(i))
            .collect()
    }

    /// Add a parent-child relationship between two lines.
    pub fn add_child(&mut self, parent: LineId, child: LineId) {
        // Prevent circular ancestry.
        debug_assert!(
            !self.is_ancestor(child, parent),
            "circular tree: child is already an ancestor of parent"
        );
        // Remove from old parent if any.
        if let Some(old_parent) = self.lines[child.0].parent {
            self.lines[old_parent.0].children.retain(|c| *c != child);
        }
        self.lines[child.0].parent = Some(parent);
        self.lines[parent.0].children.push(child);
    }

    /// Remove a child from its parent.
    pub fn remove_child(&mut self, parent: LineId, child: LineId) {
        self.lines[parent.0].children.retain(|c| *c != child);
        if self.lines[child.0].parent == Some(parent) {
            self.lines[child.0].parent = None;
        }
    }

    /// Check if `ancestor` is an ancestor (parent, grandparent, ...) of `line`.
    pub fn is_ancestor(&self, ancestor: LineId, line: LineId) -> bool {
        let mut cur = self.lines[line.0].parent;
        while let Some(pid) = cur {
            if pid == ancestor {
                return true;
            }
            cur = self.lines[pid.0].parent;
        }
        false
    }

    /// Create a new group and return its ID.
    pub fn new_group(&mut self) -> GroupId {
        let id = GroupId(self.groups.len());
        self.groups.push(Group::default());
        id
    }

    /// Add a line as a member of a group.
    pub fn group_add_member(&mut self, group: GroupId, line: LineId) {
        self.lines[line.0].group_member = Some(group);
        self.groups[group.0].members.push(line);
    }

    /// Link a line as a child of a group.
    pub fn group_add_child(&mut self, group: GroupId, line: LineId) {
        self.lines[line.0].group_child = Some(group);
        self.groups[group.0].children.push(line);
    }

    #[expect(dead_code, reason = "will be used by print engine")]
    pub(crate) fn group(&self, id: GroupId) -> &Group {
        &self.groups[id.0]
    }

    pub fn set_symbols(&mut self, symbols: Symbols) -> &mut Self {
        self.symbols = symbols;
        self
    }

    pub fn symbols(&self) -> &Symbols {
        &self.symbols
    }

    pub fn set_default_symbols(&mut self) -> &mut Self {
        self.symbols = if self.ascii {
            Symbols::ascii()
        } else {
            Symbols::unicode()
        };
        self
    }

    pub fn set_output_mode(&mut self, mode: OutputMode) -> &mut Self {
        self.output_mode = mode;
        self
    }

    pub fn output_mode(&self) -> OutputMode {
        self.output_mode
    }

    pub fn is_raw(&self) -> bool {
        self.output_mode == OutputMode::Raw
    }

    pub fn is_json(&self) -> bool {
        self.output_mode == OutputMode::Json
    }

    pub fn is_export(&self) -> bool {
        self.output_mode == OutputMode::Export
    }

    pub fn is_shellvar(&self) -> bool {
        self.output_mode == OutputMode::ShellVar
    }

    pub fn enable_colors(&mut self, enable: bool) -> &mut Self {
        self.colors_enabled = enable;
        self
    }

    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    pub fn enable_no_headings(&mut self, enable: bool) -> &mut Self {
        self.no_headings = enable;
        self
    }

    pub fn is_no_headings(&self) -> bool {
        self.no_headings
    }

    pub fn enable_header_repeat(&mut self, enable: bool) -> &mut Self {
        self.header_repeat = enable;
        self
    }

    pub fn is_header_repeat(&self) -> bool {
        self.header_repeat
    }

    pub fn enable_maxout(&mut self, enable: bool) -> &mut Self {
        self.maxout = enable;
        if enable {
            self.minout = false;
        }
        self
    }

    pub fn is_maxout(&self) -> bool {
        self.maxout
    }

    pub fn enable_minout(&mut self, enable: bool) -> &mut Self {
        self.minout = enable;
        if enable {
            self.maxout = false;
        }
        self
    }

    pub fn is_minout(&self) -> bool {
        self.minout
    }

    pub fn enable_no_wrap(&mut self, enable: bool) -> &mut Self {
        self.no_wrap = enable;
        self
    }

    pub fn is_no_wrap(&self) -> bool {
        self.no_wrap
    }

    pub fn enable_no_linesep(&mut self, enable: bool) -> &mut Self {
        self.no_linesep = enable;
        self
    }

    pub fn is_no_linesep(&self) -> bool {
        self.no_linesep
    }

    pub fn enable_no_encoding(&mut self, enable: bool) -> &mut Self {
        self.no_encoding = enable;
        self
    }

    pub fn is_no_encoding(&self) -> bool {
        self.no_encoding
    }

    pub fn enable_ascii(&mut self, enable: bool) -> &mut Self {
        self.ascii = enable;
        self.set_default_symbols();
        self
    }

    pub fn is_ascii(&self) -> bool {
        self.ascii
    }

    pub fn set_column_separator(&mut self, sep: &str) -> &mut Self {
        self.column_separator = sep.to_string();
        self
    }

    pub fn column_separator(&self) -> &str {
        &self.column_separator
    }

    pub fn set_line_separator(&mut self, sep: &str) -> &mut Self {
        self.line_separator = sep.to_string();
        self
    }

    pub fn line_separator(&self) -> &str {
        &self.line_separator
    }

    pub fn set_termforce(&mut self, force: TermForce) -> &mut Self {
        self.termforce = force;
        self
    }

    pub fn termforce(&self) -> TermForce {
        self.termforce
    }

    pub fn set_termwidth(&mut self, width: usize) -> &mut Self {
        self.termwidth = Some(width);
        self
    }

    pub fn termwidth(&self) -> usize {
        self.termwidth.unwrap_or(80)
    }

    pub fn set_termheight(&mut self, height: usize) -> &mut Self {
        self.termheight = Some(height);
        self
    }

    pub fn termheight(&self) -> usize {
        self.termheight.unwrap_or(24)
    }

    pub fn reduce_termwidth(&mut self, reduce: usize) -> &mut Self {
        self.termwidth_reduce = reduce;
        self
    }

    /// Sort lines by the given column index using the column's comparator.
    pub fn sort(&mut self, col_idx: usize) {
        self.sort_column = Some(col_idx);
        self.sort_roots(col_idx);
    }

    /// Re-sort using the previously set sort column.
    pub fn resort(&mut self) {
        if let Some(col_idx) = self.sort_column {
            self.sort_roots(col_idx);
        }
    }

    fn sort_roots(&mut self, col_idx: usize) {
        let mut root_ids = self.root_line_ids();
        self.sort_ids(&mut root_ids, col_idx);
        // Sort children recursively.
        for id in &root_ids {
            self.sort_children_recursive(*id, col_idx);
        }
    }

    fn sort_children_recursive(&mut self, parent: LineId, col_idx: usize) {
        let mut children = self.lines[parent.0].children.clone();
        self.sort_ids(&mut children, col_idx);
        self.lines[parent.0].children = children;
        let children = self.lines[parent.0].children.clone();
        for child in children {
            self.sort_children_recursive(child, col_idx);
        }
    }

    fn sort_ids(&self, ids: &mut [LineId], col_idx: usize) {
        ids.sort_by(|a, b| {
            let cell_a = self.lines[a.0].cell(col_idx);
            let cell_b = self.lines[b.0].cell(col_idx);
            match (cell_a, cell_b) {
                (Some(a), Some(b)) => {
                    if let Some(col) = self.columns.get(col_idx) {
                        col.cmp_cells(a, b)
                    } else {
                        crate::cell::cmp_cells_str(a, b)
                    }
                }
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    }

    /// Re-order lines so that children follow their parents (depth-first).
    pub fn sort_by_tree(&mut self) {
        // Collect in tree order.
        let roots = self.root_line_ids();
        let mut ordered = Vec::with_capacity(self.lines.len());
        for root in roots {
            self.collect_tree_order(root, &mut ordered);
        }
        // `ordered` now contains LineIds in display order.
        // We don't physically reorder the Vec (that would invalidate IDs);
        // instead we track the display order separately for the print engine.
        // For now this is a no-op on storage — the print engine will use
        // tree-walk order directly.
        let _ = ordered;
    }

    fn collect_tree_order(&self, id: LineId, out: &mut Vec<LineId>) {
        out.push(id);
        for &child in &self.lines[id.0].children {
            self.collect_tree_order(child, out);
        }
    }

    /// Returns true if any column has the TREE flag.
    pub fn is_tree(&self) -> bool {
        self.columns.iter().any(|c| c.is_tree())
    }

    /// Effective terminal width after reduction.
    #[expect(dead_code, reason = "will be used by print engine")]
    pub(crate) fn effective_termwidth(&self) -> usize {
        self.termwidth().saturating_sub(self.termwidth_reduce)
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
