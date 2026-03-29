/// Characters used for rendering tree hierarchies and group relationships.
#[derive(Debug, Clone)]
pub struct Symbols {
    pub branch: String,
    pub vertical: String,
    pub right: String,
    pub title_padding: String,
    pub cell_padding: String,

    pub group_vertical: String,
    pub group_horizontal: String,
    pub group_first_member: String,
    pub group_last_member: String,
    pub group_middle_member: String,
    pub group_last_child: String,
    pub group_middle_child: String,
}

impl Symbols {
    /// UTF-8 tree/group symbols (the default).
    pub fn unicode() -> Self {
        Self {
            branch: "\u{251c}\u{2500}".to_string(), // ├─
            vertical: "\u{2502} ".to_string(),      // │
            right: "\u{2514}\u{2500}".to_string(),  // └─
            title_padding: " ".to_string(),
            cell_padding: " ".to_string(),

            group_vertical: "\u{2502} ".to_string(), // │
            group_horizontal: "\u{2500}\u{2500}".to_string(), // ──
            group_first_member: "\u{250c}\u{2500}".to_string(), // ┌─
            group_last_member: "\u{2514}\u{2500}".to_string(), // └─
            group_middle_member: "\u{251c}\u{2500}".to_string(), // ├─
            group_last_child: "\u{2514}\u{2500}".to_string(), // └─
            group_middle_child: "\u{251c}\u{2500}".to_string(), // ├─
        }
    }

    /// ASCII-only fallback symbols.
    pub fn ascii() -> Self {
        Self {
            branch: "|-".to_string(),
            vertical: "| ".to_string(),
            right: "`-".to_string(),
            title_padding: " ".to_string(),
            cell_padding: " ".to_string(),

            group_vertical: "| ".to_string(),
            group_horizontal: "--".to_string(),
            group_first_member: ",-".to_string(),
            group_last_member: "`-".to_string(),
            group_middle_member: "|-".to_string(),
            group_last_child: "`-".to_string(),
            group_middle_child: "|-".to_string(),
        }
    }
}

impl Default for Symbols {
    fn default() -> Self {
        Self::unicode()
    }
}
