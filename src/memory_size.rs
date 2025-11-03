//! Types for representing the size of objects in memory.

use core::fmt::Display;

/// The size of an object in memory, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemorySize(pub usize);

impl MemorySize {
    /// Creates a new [`MemorySize`] from the given number of bytes.
    pub fn new(bytes: usize) -> Self {
        MemorySize(bytes)
    }

    /// Returns the size in bytes.
    pub fn as_bytes(&self) -> usize {
        self.0
    }

    /// Returns the size in kilobytes.
    /// 1 kilobyte = 1024 bytes.
    pub fn as_kilobytes(&self) -> f64 {
        self.0 as f64 / 1024.0
    }

    /// Returns the size in megabytes.
    /// 1 megabyte = 1024 kilobytes.
    pub fn as_megabytes(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0)
    }

    /// Returns the size in gigabytes.
    pub fn as_gigabytes(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Returns the size in terabytes.
    pub fn as_terabytes(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)
    }

    /// Determine the appropriate unit for displaying the memory size.
    ///
    /// Units are chosen such that the value is at least 1 in that unit.
    ///
    /// This is used for formatting the memory size in a human-readable way,
    /// such as in the [`Display`] implementation for this type.
    pub fn appropriate_unit(&self) -> MemoryUnit {
        if self.0 >= 1024 * 1024 * 1024 * 1024 {
            MemoryUnit::Terabytes
        } else if self.0 >= 1024 * 1024 * 1024 {
            MemoryUnit::Gigabytes
        } else if self.0 >= 1024 * 1024 {
            MemoryUnit::Megabytes
        } else if self.0 >= 1024 {
            MemoryUnit::Kilobytes
        } else {
            MemoryUnit::Bytes
        }
    }
}

impl Display for MemorySize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let unit = self.appropriate_unit();
        match unit {
            MemoryUnit::Bytes => write!(f, "{} B", self.as_bytes()),
            MemoryUnit::Kilobytes => write!(f, "{:.2} KB", self.as_kilobytes()),
            MemoryUnit::Megabytes => write!(f, "{:.2} MB", self.as_megabytes()),
            MemoryUnit::Gigabytes => write!(f, "{:.2} GB", self.as_gigabytes()),
            MemoryUnit::Terabytes => write!(f, "{:.2} TB", self.as_terabytes()),
        }
    }
}

/// Common units for representing memory size.
///
/// Used for determining the most appropriate unit to display a [`MemorySize`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryUnit {
    /// 8 bits
    Bytes,
    /// 1 kilobyte = 1024 bytes
    Kilobytes,
    /// 1 megabyte = 1024 kilobytes
    Megabytes,
    /// 1 gigabyte = 1024 megabytes
    Gigabytes,
    /// 1 terabyte = 1024 gigabytes
    Terabytes,
}
