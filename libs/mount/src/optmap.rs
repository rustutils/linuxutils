//! Classify a parsed [`OptionList`] into VFS flags, fs-specific
//! data, and userspace-only options.
//!
//! `mount(2)` takes three pieces of option-shaped state:
//!   - **VFS flags** — `MS_*` bits passed in the `flags` argument.
//!     Names like `ro`, `nosuid`, `noatime`. Toggleable: `rw`
//!     clears `MS_RDONLY`, `suid` clears `MS_NOSUID`, etc.
//!   - **fs-specific data** — a comma-separated string passed as
//!     the `data` argument and interpreted by the filesystem
//!     driver. Things like `uid=1000`, `fmask=0022`,
//!     `subvol=/home`.
//!   - **userspace-only options** — recognised by `mount(8)` and
//!     friends but never seen by the kernel. `noauto`, `_netdev`,
//!     `user`, `nofail`, anything in the `x-*` namespace.
//!
//! [`classify`] splits an `OptionList` into all three buckets in a
//! single pass, applying the toggle rules in order so `ro,rw`
//! correctly resolves to "no `MS_RDONLY`".
//!
//! The optmap below is **not** an exhaustive port of upstream's —
//! it covers the universally-recognised VFS names plus the common
//! userspace options. Anything unknown is bucketed as fs-specific
//! data; the filesystem driver decides what to do with it.

use crate::{MountFlags, OptionList};

/// Result of classifying an [`OptionList`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Classified {
    /// VFS-level flags ready for the `mount(2)` `flags` argument.
    pub flags: MountFlags,

    /// Filesystem-specific options, comma-joined, ready for the
    /// `data` argument. Empty string when no fs-specific options
    /// were present.
    pub fs_data: String,

    /// Options recognised as userspace-only (`noauto`, `_netdev`,
    /// `x-*`, ...) — these never go to the kernel, but tools may
    /// want them for orchestration (auto-mounting, fail policy,
    /// systemd unit ordering).
    pub user_options: OptionList,
}

impl Classified {
    /// `data` argument formatted for [`crate::syscall::mount`] — `None`
    /// when there's nothing fs-specific to send.
    pub fn fs_data_arg(&self) -> Option<&str> {
        if self.fs_data.is_empty() {
            None
        } else {
            Some(self.fs_data.as_str())
        }
    }
}

/// Split an option list into VFS flags / fs data / userspace bag.
///
/// Order matters: `ro,rw` ⇒ no `MS_RDONLY`, matching the kernel's
/// "later-wins" rule. Unknown option names become fs-specific data.
pub fn classify(opts: &OptionList) -> Classified {
    let mut out = Classified::default();
    let mut data_parts: Vec<String> = Vec::new();

    for opt in opts.iter() {
        if opt.name == "defaults" {
            // POSIX-ish "default mount" pseudo-option: rw, suid, dev,
            // exec, async, nouser. All of those are the *cleared*
            // state in MountFlags terms (no MS_* bit set), so for our
            // purposes `defaults` is effectively a no-op — represent
            // it by *not* touching out.flags.
            continue;
        }
        if let Some(action) = lookup_vfs(&opt.name) {
            apply_vfs(&mut out.flags, action);
            continue;
        }
        if is_userspace_only(&opt.name) {
            out.user_options.push(opt.clone());
            continue;
        }
        // Anything else: fs-specific data, passed through verbatim.
        data_parts.push(opt.to_string());
    }

    out.fs_data = data_parts.join(",");
    out
}

#[derive(Copy, Clone, Debug)]
enum Vfs {
    /// Set the named flag(s).
    Set(MountFlags),
    /// Clear the named flag(s).
    Clear(MountFlags),
}

fn apply_vfs(flags: &mut MountFlags, op: Vfs) {
    match op {
        Vfs::Set(b) => *flags |= b,
        Vfs::Clear(b) => *flags &= !b,
    }
}

fn lookup_vfs(name: &str) -> Option<Vfs> {
    use MountFlags as F;
    Some(match name {
        // Read-only / read-write
        "ro" => Vfs::Set(F::RDONLY),
        "rw" => Vfs::Clear(F::RDONLY),

        // suid / nosuid
        "nosuid" => Vfs::Set(F::NOSUID),
        "suid" => Vfs::Clear(F::NOSUID),

        // dev / nodev
        "nodev" => Vfs::Set(F::NODEV),
        "dev" => Vfs::Clear(F::NODEV),

        // exec / noexec
        "noexec" => Vfs::Set(F::NOEXEC),
        "exec" => Vfs::Clear(F::NOEXEC),

        // atime family
        "noatime" => Vfs::Set(F::NOATIME),
        "atime" => Vfs::Clear(F::NOATIME),
        "nodiratime" => Vfs::Set(F::NODIRATIME),
        "diratime" => Vfs::Clear(F::NODIRATIME),
        "relatime" => Vfs::Set(F::RELATIME),
        "norelatime" => Vfs::Clear(F::RELATIME),
        "strictatime" => Vfs::Set(F::STRICTATIME),
        "nostrictatime" => Vfs::Clear(F::STRICTATIME),
        "lazytime" => Vfs::Set(F::LAZYTIME),
        "nolazytime" => Vfs::Clear(F::LAZYTIME),

        // Write-back behaviour
        "sync" => Vfs::Set(F::SYNCHRONOUS),
        "async" => Vfs::Clear(F::SYNCHRONOUS),
        "dirsync" => Vfs::Set(F::DIRSYNC),

        // Mandatory locking
        "mand" => Vfs::Set(F::MANDLOCK),
        "nomand" => Vfs::Clear(F::MANDLOCK),

        // Misc
        "silent" => Vfs::Set(F::SILENT),
        "loud" => Vfs::Clear(F::SILENT),
        "iversion" => Vfs::Set(F::I_VERSION),
        "noiversion" => Vfs::Clear(F::I_VERSION),
        "nosymfollow" => Vfs::Set(F::NOSYMFOLLOW),
        "symfollow" => Vfs::Clear(F::NOSYMFOLLOW),

        // Operation selectors (set-only — not toggled by an inverse)
        "remount" => Vfs::Set(F::REMOUNT),
        "bind" => Vfs::Set(F::BIND),
        "rbind" => Vfs::Set(F::BIND | F::REC),

        // Propagation (set-only)
        "private" => Vfs::Set(F::PRIVATE),
        "shared" => Vfs::Set(F::SHARED),
        "slave" => Vfs::Set(F::SLAVE),
        "unbindable" => Vfs::Set(F::UNBINDABLE),
        "rprivate" => Vfs::Set(F::PRIVATE | F::REC),
        "rshared" => Vfs::Set(F::SHARED | F::REC),
        "rslave" => Vfs::Set(F::SLAVE | F::REC),
        "runbindable" => Vfs::Set(F::UNBINDABLE | F::REC),

        _ => return None,
    })
}

fn is_userspace_only(name: &str) -> bool {
    // The `x-` / `X-` prefix is the userspace-options namespace by
    // convention; everything in there is for the mount tool, never
    // for the kernel.
    if name.starts_with("x-") || name.starts_with("X-") {
        return true;
    }
    matches!(
        name,
        "noauto"
            | "auto"
            | "user"
            | "users"
            | "nouser"
            | "owner"
            | "group"
            | "_netdev"
            | "nofail"
            | "comment"
    )
}

/// Convenience: drop user-only options from an option list, leaving
/// only what should reach the kernel (VFS flags' string forms +
/// fs-specific data). Used when round-tripping options into
/// fstab-shaped strings.
#[allow(dead_code)]
pub(crate) fn strip_userspace(opts: &OptionList) -> OptionList {
    let mut out = OptionList::new();
    for o in opts.iter() {
        if !is_userspace_only(&o.name) {
            out.push(o.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify_str(s: &str) -> Classified {
        classify(&OptionList::parse(s))
    }

    #[test]
    fn empty_input_is_empty_output() {
        let r = classify_str("");
        assert_eq!(r.flags, MountFlags::empty());
        assert!(r.fs_data.is_empty());
        assert!(r.user_options.is_empty());
    }

    #[test]
    fn defaults_is_a_noop() {
        let r = classify_str("defaults");
        assert_eq!(r.flags, MountFlags::empty());
        assert!(r.fs_data.is_empty());
        assert!(r.user_options.is_empty());
    }

    #[test]
    fn vfs_flags_set_correctly() {
        let r = classify_str("ro,nosuid,nodev,noexec");
        assert_eq!(
            r.flags,
            MountFlags::RDONLY
                | MountFlags::NOSUID
                | MountFlags::NODEV
                | MountFlags::NOEXEC,
        );
    }

    #[test]
    fn later_wins_for_toggleable_flags() {
        // ro then rw: rw wins (clears RDONLY).
        let r = classify_str("ro,rw");
        assert!(!r.flags.contains(MountFlags::RDONLY));
        // rw then ro: ro wins.
        let r = classify_str("rw,ro");
        assert!(r.flags.contains(MountFlags::RDONLY));
    }

    #[test]
    fn fs_specific_data_passes_through() {
        let r = classify_str("uid=1000,gid=1000,fmask=0022");
        assert_eq!(r.flags, MountFlags::empty());
        assert_eq!(r.fs_data, "uid=1000,gid=1000,fmask=0022");
        assert!(r.user_options.is_empty());
    }

    #[test]
    fn mixed_flags_and_data() {
        let r = classify_str("ro,uid=1000,nosuid,fmask=0022");
        assert_eq!(r.flags, MountFlags::RDONLY | MountFlags::NOSUID);
        // fs-data preserves source order across only the data items.
        assert_eq!(r.fs_data, "uid=1000,fmask=0022");
    }

    #[test]
    fn userspace_only_options_bucket_separately() {
        let r = classify_str("noauto,_netdev,nofail,uid=1000");
        assert_eq!(r.flags, MountFlags::empty());
        assert_eq!(r.fs_data, "uid=1000");
        assert_eq!(
            r.user_options.to_string(),
            "noauto,_netdev,nofail"
        );
    }

    #[test]
    fn x_namespace_is_userspace_only() {
        let r = classify_str(
            "x-systemd.required-by=foo.target,x-mount.mkdir,uid=1000",
        );
        assert_eq!(
            r.user_options.to_string(),
            "x-systemd.required-by=foo.target,x-mount.mkdir"
        );
        assert_eq!(r.fs_data, "uid=1000");
    }

    #[test]
    fn bind_sets_bind_flag() {
        let r = classify_str("bind");
        assert_eq!(r.flags, MountFlags::BIND);
    }

    #[test]
    fn rbind_sets_bind_and_rec() {
        let r = classify_str("rbind");
        assert_eq!(r.flags, MountFlags::BIND | MountFlags::REC);
    }

    #[test]
    fn remount_sets_remount() {
        let r = classify_str("remount,ro");
        assert_eq!(r.flags, MountFlags::REMOUNT | MountFlags::RDONLY);
    }

    #[test]
    fn propagation_recursive_combines() {
        let r = classify_str("rshared");
        assert_eq!(r.flags, MountFlags::SHARED | MountFlags::REC);
    }

    #[test]
    fn fs_data_arg_returns_none_when_empty() {
        let r = classify_str("ro");
        assert_eq!(r.fs_data_arg(), None);
        let r = classify_str("uid=1000");
        assert_eq!(r.fs_data_arg(), Some("uid=1000"));
    }

    #[test]
    fn unknown_no_flag_is_fs_data_not_userspace() {
        // `unknownopt` isn't in our VFS map nor userspace list →
        // bucketed as fs-specific data, the filesystem driver
        // decides what to do.
        let r = classify_str("unknownopt=value");
        assert_eq!(r.flags, MountFlags::empty());
        assert_eq!(r.fs_data, "unknownopt=value");
    }

    #[test]
    fn defaults_then_ro_still_sets_rdonly() {
        // `defaults` is a no-op, so subsequent toggles still apply.
        let r = classify_str("defaults,ro,uid=1000");
        assert_eq!(r.flags, MountFlags::RDONLY);
        assert_eq!(r.fs_data, "uid=1000");
    }

    #[test]
    fn strip_userspace_removes_user_options_only() {
        let opts = OptionList::parse("ro,noauto,uid=1000,_netdev");
        let stripped = strip_userspace(&opts);
        assert_eq!(stripped.to_string(), "ro,uid=1000");
    }
}
