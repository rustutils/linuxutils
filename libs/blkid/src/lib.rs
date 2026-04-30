//! Block-device tag resolution via the udev `/dev/disk/by-*`
//! symlink tree.
//!
//! This crate is the Rust-native counterpart to a (small) subset of
//! upstream libblkid. Its purpose is to answer two questions for the
//! mount tooling:
//!
//! - **Resolve**: given a [`Tag`] like `LABEL=root` or
//!   `UUID=...`, what device path does it refer to?
//! - **Reverse-lookup**: given a device path like `/dev/sda1`, what
//!   tags does it carry?
//!
//! v0 implements both by reading the `by-label`, `by-uuid`,
//! `by-partlabel`, `by-partuuid`, and `by-id` subdirectories of
//! `/dev/disk/`. Those symlinks are populated at boot by udev rules
//! that consume real upstream blkid's filesystem-signature probe
//! output. Piggy-backing on them gets us correct tag resolution for
//! any mounted filesystem on a normal system without reimplementing
//! the probe layer.
//!
//! What's deliberately **not** here:
//!
//! - Filesystem-signature probing (`blkid -p /dev/sda1`).
//!   Reimplementing the hundreds of fs/partition/RAID detectors
//!   upstream supports is a multi-month effort; deferred to a later
//!   crate slice.
//! - Cache files (`/run/blkid/blkid.tab`).  Our index is the udev
//!   symlink tree itself, queried on demand or eagerly cached in
//!   memory.
//!
//! See `libs/blkid/README.md` for the status table.

pub mod cache;

pub use cache::{Cache, ResolveError};
pub use linuxutils_common::{Tag, TagKind};
