pub mod man;
pub mod tag;

pub use tag::{Tag, TagKind};

/// Cap for `--help` output width. Tools should set
/// `#[command(max_term_width = HELP_WIDTH_MAX)]` so help text wraps
/// consistently in wide terminals (and produces stable snapshots).
pub const HELP_WIDTH_MAX: usize = 100;
