#[cfg(feature = "lsmem")]
mod sysfs;

#[cfg(feature = "blkdiscard")]
pub mod blkdiscard;

#[cfg(feature = "choom")]
pub mod choom;

#[cfg(feature = "ctrlaltdel")]
pub mod ctrlaltdel;

#[cfg(feature = "dmesg")]
pub mod dmesg;

#[cfg(feature = "eject")]
pub mod eject;

#[cfg(feature = "fadvise")]
pub mod fadvise;

#[cfg(feature = "fallocate")]
pub mod fallocate;

#[cfg(feature = "flock")]
pub mod flock;

#[cfg(feature = "fsfreeze")]
pub mod fsfreeze;

#[cfg(feature = "fstrim")]
pub mod fstrim;

#[cfg(feature = "ipcmk")]
pub mod ipcmk;

#[cfg(feature = "ipcrm")]
pub mod ipcrm;

#[cfg(feature = "ipcs")]
pub mod ipcs;

#[cfg(feature = "losetup")]
pub mod losetup;

#[cfg(feature = "lscpu")]
pub mod lscpu;

#[cfg(feature = "lsipc")]
pub mod lsipc;

#[cfg(feature = "lsirq")]
pub mod lsirq;

#[cfg(feature = "lsmem")]
pub mod lsmem;

#[cfg(feature = "lsns")]
pub mod lsns;

#[cfg(feature = "mountpoint")]
pub mod mountpoint;

#[cfg(feature = "nsenter")]
pub mod nsenter;

#[cfg(feature = "pipesz")]
pub mod pipesz;

#[cfg(feature = "pivot_root")]
pub mod pivot_root;

#[cfg(feature = "prlimit")]
pub mod prlimit;

#[cfg(feature = "renice")]
pub mod renice;

#[cfg(feature = "rfkill")]
pub mod rfkill;

#[cfg(feature = "setpgid")]
pub mod setpgid;

#[cfg(feature = "setsid")]
pub mod setsid;

#[cfg(feature = "swapoff")]
pub mod swapoff;

#[cfg(feature = "swapon")]
pub mod swapon;

#[cfg(feature = "unshare")]
pub mod unshare;
