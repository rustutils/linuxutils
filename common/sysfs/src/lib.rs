//! Helpers for reading Linux sysfs attributes.
//!
//! Sysfs exposes kernel objects as directories with plain-text attribute
//! files. This crate provides a thin typed API for reading them.
//!
//! ```no_run
//! use linuxutils_common_sysfs::SysfsDevice;
//!
//! let dev = SysfsDevice::new("/sys/devices/system/memory/memory0");
//! let state: String = dev.read_attr("state").unwrap();
//! let removable: bool = dev.read_attr_bool("removable").unwrap();
//! let index: u64 = dev.read_attr_hex("phys_index").unwrap();
//! ```

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

/// A handle to a sysfs directory (device, subsystem, etc.).
#[derive(Debug, Clone)]
pub struct SysfsDevice {
    path: PathBuf,
}

/// Errors from sysfs attribute reads.
#[derive(Debug)]
pub enum SysfsError {
    Io(io::Error),
    Parse(String),
}

impl fmt::Display for SysfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SysfsError::Io(e) => write!(f, "sysfs I/O error: {e}"),
            SysfsError::Parse(msg) => write!(f, "sysfs parse error: {msg}"),
        }
    }
}

impl std::error::Error for SysfsError {}

impl From<io::Error> for SysfsError {
    fn from(e: io::Error) -> Self {
        SysfsError::Io(e)
    }
}

impl SysfsDevice {
    /// Create a handle to the given sysfs directory.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The filesystem path of this device.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read a raw attribute value as a trimmed string.
    pub fn read_attr(&self, name: &str) -> Result<String, SysfsError> {
        let content = fs::read_to_string(self.path.join(name))?;
        Ok(content.trim().to_string())
    }

    /// Read an attribute and parse it with `FromStr`.
    pub fn read_attr_parse<T: FromStr>(
        &self,
        name: &str,
    ) -> Result<T, SysfsError>
    where
        T::Err: fmt::Display,
    {
        let s = self.read_attr(name)?;
        s.parse::<T>()
            .map_err(|e| SysfsError::Parse(format!("{name}: {e}")))
    }

    /// Read an attribute as a boolean (`"1"` / `"0"` or `"yes"` / `"no"`).
    pub fn read_attr_bool(&self, name: &str) -> Result<bool, SysfsError> {
        let s = self.read_attr(name)?;
        match s.as_str() {
            "1" | "yes" => Ok(true),
            "0" | "no" => Ok(false),
            _ => Err(SysfsError::Parse(format!(
                "{name}: expected 0/1 or yes/no, got {s:?}"
            ))),
        }
    }

    /// Read an attribute as a hexadecimal u64.
    pub fn read_attr_hex(&self, name: &str) -> Result<u64, SysfsError> {
        let s = self.read_attr(name)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        u64::from_str_radix(s, 16)
            .map_err(|e| SysfsError::Parse(format!("{name}: {e}")))
    }

    /// Check whether a named attribute file exists.
    pub fn has_attr(&self, name: &str) -> bool {
        self.path.join(name).exists()
    }

    /// Open a child device directory.
    pub fn child(&self, name: &str) -> Self {
        Self::new(self.path.join(name))
    }

    /// List child directories matching a prefix (e.g., `"memory"` lists
    /// `memory0`, `memory1`, ...).
    pub fn children_with_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<Self>, SysfsError> {
        let mut children = Vec::new();
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str()
                && name.starts_with(prefix)
                && entry.file_type()?.is_dir()
            {
                children.push(Self::new(entry.path()));
            }
        }
        children.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(children)
    }
}
