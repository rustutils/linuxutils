//! Rust-native types and parsers for Linux mount tooling.
//!
//! This crate is the data model and parsing layer that mount-related
//! tools (`findmnt`, `mount`, `umount`, ...) build on. It is *not* an
//! ABI-compatible reimplementation of `libmount.so` — that lives in
//! `compat/libmount/` as a separate cdylib facade.
//!
//! The v0 surface (this commit) is the leaf types: [`Spec`] / [`Tag`]
//! for source identifiers, [`OptionList`] for parsed mount option
//! strings, and [`MountFlags`] for the `MS_*` kernel flag bits. The
//! mount-table model and parsers (`mountinfo`, `fstab`, `swaps`) land
//! in subsequent slices.

pub mod entry;
pub mod flags;
pub mod mountinfo;
pub mod options;
pub mod spec;
pub mod table;

pub use entry::{MountEntry, Propagation};
pub use flags::MountFlags;
pub use options::{MountOption, OptionList};
pub use spec::{Spec, Tag, TagKind};
pub use table::MountTable;
