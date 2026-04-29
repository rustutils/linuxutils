//! Per-tool man page content shared with the man-page generator.
//!
//! `clap_mangen` produces NAME / SYNOPSIS / OPTIONS automatically from
//! the clap derive. Anything richer — DESCRIPTION prose, EXAMPLES,
//! NOTES, FILES, SEE ALSO — needs to be authored by hand and stitched
//! into the rendered output by the generator.
//!
//! Each tool exposes a `pub const MAN: ManContent` describing what to
//! splice in. Bodies are raw groff (`.SH`, `.TP`, `.B`, `.I`, etc.) so
//! the generator can write them verbatim into the man page between the
//! auto-generated sections.

#[derive(Debug)]
pub struct ManContent {
    /// Replaces clap's auto-generated DESCRIPTION (which would otherwise
    /// be the short `about` string from the derive). Plain text — the
    /// generator wraps it in `.SH DESCRIPTION` itself.
    pub description: Option<&'static str>,

    /// Sections appended after OPTIONS, in the order given. Each entry
    /// is `(uppercase title, raw groff body)`. Conventional ordering:
    /// EXAMPLES, FILES, ENVIRONMENT, NOTES, BUGS, SEE ALSO.
    pub extra_sections: &'static [(&'static str, &'static str)],
}

impl ManContent {
    pub const fn empty() -> Self {
        Self {
            description: None,
            extra_sections: &[],
        }
    }
}
