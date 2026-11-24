//! Per-tool man page content shared with the man-page generator.
//!
//! `clap_mangen` produces NAME, SYNOPSIS and OPTIONS automatically from
//! the clap derive. Anything richer (DESCRIPTION prose, EXAMPLES,
//! NOTES, FILES, SEE ALSO) needs to be authored by hand and stitched
//! into the rendered output by the generator.
//!
//! Each tool exposes a `pub const MAN: ManContent` describing what to
//! splice in. Bodies are raw groff (`.SH`, `.TP`, `.B`, `.I`, etc.) so
//! the generator can write them verbatim into the man page between the
//! auto-generated sections.

#[derive(Debug)]
pub struct ManContent {
    /// Replaces clap's auto-generated DESCRIPTION (which would otherwise
    /// be the short `about` string from the derive). Plain text. The
    /// generator wraps it in `.SH DESCRIPTION` itself.
    pub description: Option<&'static str>,

    /// Sections appended after OPTIONS, in the order given. Each entry
    /// is `(uppercase title, raw groff body)`. Conventional ordering:
    /// EXAMPLES, FILES, ENVIRONMENT, NOTES, BUGS, SEE ALSO.
    pub extra_sections: &'static [(&'static str, &'static str)],

    /// Manual section, matching upstream: 1 for user commands, 8 for
    /// system administration commands.
    pub section: u8,
}

impl ManContent {
    /// No hand-written content, in section 1.
    pub const fn empty() -> Self {
        Self {
            description: None,
            extra_sections: &[],
            section: 1,
        }
    }

    /// The same content in another manual section.
    pub const fn section(self, section: u8) -> Self {
        Self { section, ..self }
    }
}
