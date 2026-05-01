//! Wrappers around `mount(2)` and `umount2(2)`.
//!
//! Built on top of [`rustix::mount`]; this module just translates
//! between our [`MountFlags`] and rustix's, picks the right flavour
//! of `mount` syscall (regular, remount, bind, move, propagation
//! change), and surfaces a `std::io::Error` so callers don't need
//! to depend on rustix's error type.

use crate::MountFlags;
use bitflags::bitflags;
use rustix::mount as rustix_mount;
use std::{ffi::CString, io, path::Path};

bitflags! {
    /// Flags accepted by `umount2(2)`.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UnmountFlags: u32 {
        /// `MNT_FORCE` — abort pending I/O on dying filesystems.
        const FORCE    = 1;
        /// `MNT_DETACH` — lazy unmount; detach now, free when idle.
        const DETACH   = 2;
        /// `MNT_EXPIRE` — mark for next-call expiry; for inactive
        /// filesystems.
        const EXPIRE   = 4;
        /// `UMOUNT_NOFOLLOW` — don't follow a symlink at the target.
        const NOFOLLOW = 8;
    }
}

fn to_rustix_mount_flags(f: MountFlags) -> rustix_mount::MountFlags {
    let bits = f.bits();
    // The upstream `MS_*` constants we expose all fit in `u32`. If a
    // future extension adds a 64-bit-only bit we want a loud failure
    // here rather than a silent truncation.
    debug_assert_eq!(
        bits >> 32,
        0,
        "MountFlags has bits above bit 31 that don't fit a u32 syscall arg"
    );
    rustix_mount::MountFlags::from_bits_retain(bits as u32)
}

fn to_rustix_unmount_flags(f: UnmountFlags) -> rustix_mount::UnmountFlags {
    rustix_mount::UnmountFlags::from_bits_retain(f.bits())
}

fn to_rustix_propagation_flags(
    f: MountFlags,
) -> rustix_mount::MountPropagationFlags {
    // The propagation bits we use (SHARED/SLAVE/PRIVATE/UNBINDABLE +
    // optional REC) live in MountFlags for this crate; rustix
    // separates them into MountPropagationFlags. Same uapi values, so
    // a mask-and-cast works.
    let bits = f.bits() as u32;
    rustix_mount::MountPropagationFlags::from_bits_retain(bits)
}

/// `mount(source, target, fstype, flags, data)` — fresh mount.
///
/// `data` is the filesystem-specific options string the kernel
/// passes to the fs driver verbatim (e.g. `uid=1000,fmask=0022`).
/// Pass `None` for fs types that don't take any data.
pub fn mount(
    source: &Path,
    target: &Path,
    fstype: &str,
    flags: MountFlags,
    data: Option<&str>,
) -> io::Result<()> {
    let result = match data {
        Some(d) => {
            let cd = CString::new(d)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            rustix_mount::mount(
                source,
                target,
                fstype,
                to_rustix_mount_flags(flags),
                Some(cd.as_c_str()),
            )
        }
        None => rustix_mount::mount(
            source,
            target,
            fstype,
            to_rustix_mount_flags(flags),
            None,
        ),
    };
    result.map_err(io::Error::from)
}

/// `mount(NULL, target, NULL, MS_REMOUNT | flags, data)` — alter the
/// flags and/or fs-data of an already-mounted filesystem.
pub fn remount(
    target: &Path,
    flags: MountFlags,
    data: Option<&str>,
) -> io::Result<()> {
    let cd = match data {
        Some(d) => Some(
            CString::new(d)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        ),
        None => None,
    };
    let data_arg: &str =
        cd.as_deref().and_then(|c| c.to_str().ok()).unwrap_or("");
    rustix_mount::mount_remount(target, to_rustix_mount_flags(flags), data_arg)
        .map_err(io::Error::from)
}

/// `mount(source, target, NULL, MS_BIND, NULL)` — bind-mount an
/// existing tree.
pub fn bind(source: &Path, target: &Path) -> io::Result<()> {
    rustix_mount::mount_bind(source, target).map_err(io::Error::from)
}

/// `mount(source, target, NULL, MS_BIND | MS_REC, NULL)` — recursive
/// bind, includes submounts.
pub fn rbind(source: &Path, target: &Path) -> io::Result<()> {
    rustix_mount::mount_bind_recursive(source, target).map_err(io::Error::from)
}

/// `mount(NULL, target, NULL, prop_flags, NULL)` — change
/// propagation type (`MS_SHARED` / `MS_SLAVE` / `MS_PRIVATE` /
/// `MS_UNBINDABLE`, optionally combined with `MS_REC`).
pub fn change_propagation(target: &Path, flags: MountFlags) -> io::Result<()> {
    rustix_mount::mount_change(target, to_rustix_propagation_flags(flags))
        .map_err(io::Error::from)
}

/// `mount(source, target, NULL, MS_MOVE, NULL)` — atomically move a
/// mount tree.
pub fn move_mount(source: &Path, target: &Path) -> io::Result<()> {
    rustix_mount::mount_move(source, target).map_err(io::Error::from)
}

/// `umount2(target, flags)` — unmount.
pub fn umount(target: &Path, flags: UnmountFlags) -> io::Result<()> {
    rustix_mount::unmount(target, to_rustix_unmount_flags(flags))
        .map_err(io::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spot-check that our `UnmountFlags` bit values match the kernel
    /// uapi constants (and hence rustix's).
    #[test]
    fn unmount_flag_bits_match_uapi() {
        assert_eq!(UnmountFlags::FORCE.bits(), 1);
        assert_eq!(UnmountFlags::DETACH.bits(), 2);
        assert_eq!(UnmountFlags::EXPIRE.bits(), 4);
        assert_eq!(UnmountFlags::NOFOLLOW.bits(), 8);
    }

    #[test]
    fn flag_translation_round_trip() {
        // No bits to translate above 31, so converting our
        // MountFlags down to rustix's u32-shaped bitflags should
        // preserve the bit pattern.
        let ours =
            MountFlags::RDONLY | MountFlags::NOSUID | MountFlags::NOATIME;
        let translated = to_rustix_mount_flags(ours);
        assert_eq!(translated.bits() as u64, ours.bits());
    }
}
