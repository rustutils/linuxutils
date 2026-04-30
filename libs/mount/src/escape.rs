//! Octal-escape decoding shared by the mountinfo and fstab parsers.
//!
//! Both formats encode whitespace and backslash inside path-like
//! fields as `\NNN` (three octal digits). This is the inverse of
//! libmount's `mnt_mangle()`.
//!
//! The decoder is byte-in/byte-out so it can preserve non-UTF-8
//! filesystem labels and paths verbatim. Lone backslashes or runs
//! shorter than three octal digits pass through unchanged, matching
//! what the kernel and `mount(8)` produce.

/// Decode `\NNN` octal escapes, leaving everything else verbatim.
pub(crate) fn unmangle(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\\' && i + 3 < s.len() {
            if let (Some(a), Some(b), Some(c)) = (
                octal_digit(s[i + 1]),
                octal_digit(s[i + 2]),
                octal_digit(s[i + 3]),
            ) {
                out.push(a * 64 + b * 8 + c);
                i += 4;
                continue;
            }
        }
        out.push(s[i]);
        i += 1;
    }
    out
}

fn octal_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'7' => Some(b - b'0'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_space_tab_newline_backslash() {
        assert_eq!(
            unmangle(b"a\\040b\\011c\\012d\\134e"),
            b"a b\tc\nd\\e"
        );
    }

    #[test]
    fn lone_backslash_passes_through() {
        // No three octal digits following → keep verbatim.
        assert_eq!(unmangle(b"a\\b"), b"a\\b");
        assert_eq!(unmangle(b"a\\"), b"a\\");
    }

    #[test]
    fn non_octal_after_backslash_passes_through() {
        // \999 is not octal — emit the bytes literally.
        assert_eq!(unmangle(b"a\\999b"), b"a\\999b");
    }

    #[test]
    fn truncated_escape_at_end_passes_through() {
        // Need exactly three digits; two-digit run is not enough.
        assert_eq!(unmangle(b"x\\04"), b"x\\04");
    }

    #[test]
    fn preserves_non_ascii_bytes() {
        let raw: &[u8] = b"\xff\xfe\\040\xff";
        assert_eq!(unmangle(raw), b"\xff\xfe \xff");
    }

    #[test]
    fn empty_input() {
        assert_eq!(unmangle(b""), b"");
    }
}
