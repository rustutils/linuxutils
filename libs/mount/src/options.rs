//! Parsed mount option strings.
//!
//! Mount options live as comma-separated `name` or `name=value` pairs
//! in fstab, mountinfo, and on the `mount(8)` command line. This
//! module models them as an ordered list of [`MountOption`] entries —
//! ordered, because semantics depend on the position (`ro,rw` and
//! `rw,ro` mean different things), and entries can repeat.
//!
//! The classification of each option (kernel `MS_*` flag, fs-specific
//! data, userspace-only) lives in a separate optmap layer (not in v0).

use std::fmt;

/// A single mount option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountOption {
    pub name: String,
    pub value: Option<String>,
}

impl MountOption {
    pub fn flag(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }

    pub fn kv(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Some(value.into()),
        }
    }
}

impl fmt::Display for MountOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(v) => write!(f, "{}={}", self.name, v),
            None => f.write_str(&self.name),
        }
    }
}

/// An ordered list of [`MountOption`] entries.
///
/// Preserves insertion order and allows duplicate names — matches
/// kernel/`mount(8)` behaviour where later occurrences override
/// earlier ones (`ro,rw` ⇒ effectively `rw`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OptionList(Vec<MountOption>);

impl OptionList {
    /// An empty option list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a comma-separated mount option string.
    ///
    /// Empty entries (from `,,` or leading/trailing commas) are
    /// dropped. The empty string parses to an empty list. Quoting and
    /// escaping are *not* handled here — callers writing mountinfo
    /// content already work in the `mnt_unmangle`'d form.
    pub fn parse(s: &str) -> Self {
        let mut list = Vec::new();
        for part in s.split(',') {
            if part.is_empty() {
                continue;
            }
            list.push(match part.split_once('=') {
                Some((n, v)) => MountOption::kv(n, v),
                None => MountOption::flag(part),
            });
        }
        Self(list)
    }

    pub fn as_slice(&self) -> &[MountOption] {
        &self.0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, MountOption> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Append an option to the end of the list.
    pub fn push(&mut self, opt: MountOption) {
        self.0.push(opt);
    }

    /// Last value for an option name, mirroring kernel "later wins"
    /// semantics. Returns `Some(&None)` for a name that exists as a
    /// bare flag (`ro`), or `Some(&Some(value))` for `name=value`.
    pub fn last_value(&self, name: &str) -> Option<&Option<String>> {
        self.0
            .iter()
            .rev()
            .find(|o| o.name == name)
            .map(|o| &o.value)
    }

    /// Whether the option name appears at least once.
    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|o| o.name == name)
    }

    /// Remove all occurrences of an option name. Returns the number
    /// removed.
    pub fn remove_all(&mut self, name: &str) -> usize {
        let before = self.0.len();
        self.0.retain(|o| o.name != name);
        before - self.0.len()
    }
}

impl fmt::Display for OptionList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for o in &self.0 {
            if !first {
                f.write_str(",")?;
            }
            first = false;
            write!(f, "{o}")?;
        }
        Ok(())
    }
}

impl From<Vec<MountOption>> for OptionList {
    fn from(v: Vec<MountOption>) -> Self {
        Self(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag(n: &str) -> MountOption {
        MountOption::flag(n)
    }
    fn kv(n: &str, v: &str) -> MountOption {
        MountOption::kv(n, v)
    }

    #[test]
    fn parses_empty() {
        assert_eq!(OptionList::parse(""), OptionList::new());
    }

    #[test]
    fn parses_single_flag() {
        assert_eq!(OptionList::parse("ro").as_slice(), &[flag("ro")]);
    }

    #[test]
    fn parses_kv_pair() {
        assert_eq!(
            OptionList::parse("uid=1000").as_slice(),
            &[kv("uid", "1000")]
        );
    }

    #[test]
    fn parses_typical_mountinfo_options() {
        let l = OptionList::parse("rw,relatime,fmask=0022,dmask=0022");
        assert_eq!(
            l.as_slice(),
            &[
                flag("rw"),
                flag("relatime"),
                kv("fmask", "0022"),
                kv("dmask", "0022"),
            ]
        );
    }

    #[test]
    fn drops_empty_entries() {
        // Spurious commas are tolerated.
        assert_eq!(
            OptionList::parse(",ro,,rw,").as_slice(),
            &[flag("ro"), flag("rw")]
        );
    }

    #[test]
    fn empty_value_is_kv_with_empty_string() {
        // `name=` is a kv pair with empty value, distinct from a bare flag.
        assert_eq!(OptionList::parse("foo=").as_slice(), &[kv("foo", "")]);
    }

    #[test]
    fn value_can_contain_equals() {
        // Only the first `=` separates name from value, matching
        // mount(8). `foo=a=b` ⇒ `foo` = `"a=b"`.
        assert_eq!(
            OptionList::parse("foo=a=b").as_slice(),
            &[kv("foo", "a=b")]
        );
    }

    #[test]
    fn last_value_picks_latest() {
        let l = OptionList::parse("ro,rw,ro");
        // `ro` appears at index 0 and 2, both as flags (None value).
        assert_eq!(l.last_value("ro"), Some(&None));
        // Mixed: `uid=0,uid=1000,uid=500` should yield 500.
        let l = OptionList::parse("uid=0,uid=1000,uid=500");
        assert_eq!(l.last_value("uid"), Some(&Some("500".to_string())));
    }

    #[test]
    fn contains_and_remove() {
        let mut l = OptionList::parse("ro,nosuid,nodev");
        assert!(l.contains("nosuid"));
        assert!(!l.contains("rw"));
        assert_eq!(l.remove_all("nosuid"), 1);
        assert!(!l.contains("nosuid"));
        assert_eq!(l.to_string(), "ro,nodev");
    }

    #[test]
    fn round_trip_via_display() {
        let s = "rw,relatime,fmask=0022,dmask=0022,uid=1000";
        assert_eq!(OptionList::parse(s).to_string(), s);
    }

    #[test]
    fn duplicates_preserved_in_order() {
        let l = OptionList::parse("ro,rw,ro,rw");
        assert_eq!(l.len(), 4);
        assert_eq!(l.to_string(), "ro,rw,ro,rw");
    }
}
