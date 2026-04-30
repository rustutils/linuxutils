//! Kernel `MS_*` mount flags.
//!
//! These are the bits accepted by the `mount(2)` syscall's `flags`
//! argument. They model the *VFS-level* options — generic things the
//! kernel knows about regardless of filesystem type. Filesystem-specific
//! options (e.g. ext4's `barrier=1`) travel via the `data` argument
//! and aren't represented here.
//!
//! Values mirror `<sys/mount.h>` exactly so the bit pattern is the
//! same one that ships to the syscall.

use bitflags::bitflags;

bitflags! {
    /// VFS mount flags passed to `mount(2)`.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MountFlags: u64 {
        /// `MS_RDONLY` — mount read-only.
        const RDONLY        = 1;
        /// `MS_NOSUID` — ignore suid and sgid bits.
        const NOSUID        = 1 << 1;
        /// `MS_NODEV` — disallow access to device special files.
        const NODEV         = 1 << 2;
        /// `MS_NOEXEC` — disallow program execution.
        const NOEXEC        = 1 << 3;
        /// `MS_SYNCHRONOUS` — writes are synced at once.
        const SYNCHRONOUS   = 1 << 4;
        /// `MS_REMOUNT` — alter flags of a mounted filesystem.
        const REMOUNT       = 1 << 5;
        /// `MS_MANDLOCK` — allow mandatory locks.
        const MANDLOCK      = 1 << 6;
        /// `MS_DIRSYNC` — directory modifications are synchronous.
        const DIRSYNC       = 1 << 7;
        /// `MS_NOSYMFOLLOW` — don't follow symlinks.
        const NOSYMFOLLOW   = 1 << 8;
        /// `MS_NOATIME` — do not update access times.
        const NOATIME       = 1 << 10;
        /// `MS_NODIRATIME` — don't update directory access times.
        const NODIRATIME    = 1 << 11;
        /// `MS_BIND` — mount existing tree elsewhere as well.
        const BIND          = 1 << 12;
        /// `MS_MOVE` — atomically move the tree.
        const MOVE          = 1 << 13;
        /// `MS_REC` — recursive (combine with BIND/propagation flags).
        const REC           = 1 << 14;
        /// `MS_SILENT` — don't emit certain kernel messages.
        const SILENT        = 1 << 15;
        /// `MS_UNBINDABLE` — make unbindable.
        const UNBINDABLE    = 1 << 17;
        /// `MS_PRIVATE` — make private.
        const PRIVATE       = 1 << 18;
        /// `MS_SLAVE` — make slave.
        const SLAVE         = 1 << 19;
        /// `MS_SHARED` — make shared.
        const SHARED        = 1 << 20;
        /// `MS_RELATIME` — update atime relative to mtime/ctime.
        const RELATIME      = 1 << 21;
        /// `MS_I_VERSION` — update the inode I_version field.
        const I_VERSION     = 1 << 23;
        /// `MS_STRICTATIME` — always perform atime updates.
        const STRICTATIME   = 1 << 24;
        /// `MS_LAZYTIME` — update on-disk [acm]times lazily.
        const LAZYTIME      = 1 << 25;

        /// Convenience: the propagation flag set
        /// (`SHARED | SLAVE | UNBINDABLE | PRIVATE`).
        const PROPAGATION = Self::SHARED.bits()
                          | Self::SLAVE.bits()
                          | Self::UNBINDABLE.bits()
                          | Self::PRIVATE.bits();

        /// Convenience: the `mount -o secure` defaults
        /// (`NOEXEC | NOSUID | NODEV`).
        const SECURE = Self::NOEXEC.bits()
                     | Self::NOSUID.bits()
                     | Self::NODEV.bits();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spot-check that bit values match `<sys/mount.h>` so a flag set
    /// passes through to `mount(2)` unchanged.
    #[test]
    fn bit_values_match_uapi() {
        assert_eq!(MountFlags::RDONLY.bits(), 1);
        assert_eq!(MountFlags::NOSUID.bits(), 2);
        assert_eq!(MountFlags::NODEV.bits(), 4);
        assert_eq!(MountFlags::NOEXEC.bits(), 8);
        assert_eq!(MountFlags::SYNCHRONOUS.bits(), 16);
        assert_eq!(MountFlags::REMOUNT.bits(), 32);
        assert_eq!(MountFlags::MANDLOCK.bits(), 64);
        assert_eq!(MountFlags::DIRSYNC.bits(), 128);
        assert_eq!(MountFlags::NOSYMFOLLOW.bits(), 256);
        assert_eq!(MountFlags::NOATIME.bits(), 0x400);
        assert_eq!(MountFlags::NODIRATIME.bits(), 0x800);
        assert_eq!(MountFlags::BIND.bits(), 0x1000);
        assert_eq!(MountFlags::MOVE.bits(), 0x2000);
        assert_eq!(MountFlags::REC.bits(), 0x4000);
        assert_eq!(MountFlags::SILENT.bits(), 0x8000);
        assert_eq!(MountFlags::UNBINDABLE.bits(), 1 << 17);
        assert_eq!(MountFlags::PRIVATE.bits(), 1 << 18);
        assert_eq!(MountFlags::SLAVE.bits(), 1 << 19);
        assert_eq!(MountFlags::SHARED.bits(), 1 << 20);
        assert_eq!(MountFlags::RELATIME.bits(), 1 << 21);
    }

    #[test]
    fn propagation_set_is_correct() {
        assert_eq!(
            MountFlags::PROPAGATION,
            MountFlags::SHARED
                | MountFlags::SLAVE
                | MountFlags::UNBINDABLE
                | MountFlags::PRIVATE
        );
    }

    #[test]
    fn secure_set_is_correct() {
        assert_eq!(
            MountFlags::SECURE,
            MountFlags::NOEXEC | MountFlags::NOSUID | MountFlags::NODEV
        );
    }

    #[test]
    fn flags_compose() {
        let f = MountFlags::RDONLY | MountFlags::NOSUID;
        assert!(f.contains(MountFlags::RDONLY));
        assert!(f.contains(MountFlags::NOSUID));
        assert!(!f.contains(MountFlags::NOEXEC));
    }
}
