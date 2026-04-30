//! Per-mount data — one row of a mount table.

use crate::{OptionList, Spec};
use std::path::PathBuf;

/// A single mount table entry.
///
/// Same shape regardless of whether the source is `/proc/self/mountinfo`,
/// `/etc/fstab`, `/proc/swaps`, or `/run/mount/utab` — fields not
/// applicable to a given source remain at their default (typically
/// `None` or empty).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MountEntry {
    /// Mount source: device path, NFS export, bind source, or
    /// unresolved tag. Mountinfo only produces [`Spec::Path`] (the
    /// kernel resolved tags at mount time); fstab can produce either.
    pub source: Spec,

    /// Mount point.
    pub target: PathBuf,

    /// Filesystem type (`ext4`, `tmpfs`, `proc`, ...).
    pub fstype: String,

    /// Combined option list. Always populated.
    ///
    /// For mountinfo entries this is `vfs_options` and `fs_options`
    /// merged (in that order), matching upstream's
    /// `mnt_fs_get_options()`. For fstab/swaps it's the raw fourth
    /// column, undifferentiated.
    pub options: OptionList,

    /// VFS-level options as the kernel reports them in mountinfo
    /// field 6. `None` for fstab/swaps where no split exists at this
    /// layer.
    pub vfs_options: Option<OptionList>,

    /// Filesystem-specific options from mountinfo field 11. `None`
    /// for fstab/swaps.
    pub fs_options: Option<OptionList>,

    // -- mountinfo-only --
    /// Unique mount ID within the mount namespace.
    pub mount_id: Option<i32>,

    /// Mount ID of the parent mount, or self for the root.
    pub parent_id: Option<i32>,

    /// `(major, minor)` of the device hosting the filesystem.
    pub devno: Option<(u32, u32)>,

    /// Path within the filesystem that's mounted as a subtree
    /// (typically `/`, but can differ for bind mounts).
    pub root: Option<PathBuf>,

    /// Propagation state — shared/slave/unbindable, with peer group
    /// IDs where applicable. Decoded view of [`Self::optional_fields`].
    pub propagation: Propagation,

    /// Raw mountinfo optional-fields string (the space-separated tags
    /// between field 6 and the literal `-` separator), preserved
    /// verbatim. Mirrors `mnt_fs_get_optional_fields()` in upstream
    /// libmount and feeds findmnt's `OPT-FIELDS` column.
    ///
    /// Future kernel versions may add propagation tags this crate's
    /// [`Propagation`] type doesn't yet decode; storing the raw form
    /// lets us round-trip them losslessly even when the structured
    /// view is incomplete. `None` for non-mountinfo sources, and
    /// `Some(String::new())` for mountinfo entries that had no
    /// optional fields at all.
    pub optional_fields: Option<String>,

    // -- fstab-only --
    /// `dump(8)` frequency hint (column 5). `None` outside fstab.
    pub freq: Option<u32>,

    /// `fsck(8)` pass number (column 6). `None` outside fstab.
    pub passno: Option<u32>,

    // -- swaps-only (`/proc/swaps`) --
    /// Swap area type, either `"partition"` or `"file"`. `None`
    /// outside `/proc/swaps`. Mirrors `mnt_fs_get_swaptype()`.
    pub swap_type: Option<String>,

    /// Swap area size in bytes. `None` outside `/proc/swaps`. Mirrors
    /// `mnt_fs_get_size()` — kernel reports in 1 KiB units, we
    /// convert to bytes at parse time.
    pub swap_size: Option<u64>,

    /// Swap bytes currently in use. `None` outside `/proc/swaps`.
    /// Mirrors `mnt_fs_get_usedsize()`. Bytes (KiB-converted).
    pub swap_used: Option<u64>,

    /// Swap priority. Higher = preferred. `None` outside
    /// `/proc/swaps`. Mirrors `mnt_fs_get_priority()`.
    pub swap_priority: Option<i32>,
}

/// Mount propagation state.
///
/// A mount can be private (the default; all fields here zeroed),
/// shared, slave, or unbindable. Note that *shared and slave can
/// co-exist* — a mount that is `shared:N master:M` is both — so this
/// is modelled as a struct rather than an enum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Propagation {
    /// Peer group ID if this mount is shared (`shared:N`).
    pub shared: Option<u64>,

    /// Peer group ID this mount is slaved to (`master:N`).
    pub master: Option<u64>,

    /// Peer group ID this mount received its propagation from
    /// (`propagate_from:N`); only meaningful alongside `master`.
    pub propagate_from: Option<u64>,

    /// `unbindable` flag — mutually exclusive with shared/slave in
    /// practice (the kernel won't emit them together).
    pub unbindable: bool,
}

impl Propagation {
    /// True iff no propagation field is set — i.e. the default
    /// `MS_PRIVATE` state.
    pub fn is_private(&self) -> bool {
        self.shared.is_none()
            && self.master.is_none()
            && self.propagate_from.is_none()
            && !self.unbindable
    }
}
