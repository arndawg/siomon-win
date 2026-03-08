//! Low-level x86 I/O port access via `/dev/port`.
//!
//! Provides a safe wrapper around the Linux `/dev/port` character device,
//! which allows userspace programs to read and write individual I/O port
//! bytes. Requires root or `CAP_SYS_RAWIO`.
//!
//! Used for direct Super I/O chip access (Nuvoton NCT67xx, ITE IT87xx, etc.).

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

/// Handle for I/O port access via `/dev/port`.
///
/// Keeps the file descriptor open for the lifetime of the struct to avoid
/// repeated open/close overhead on every register read.
pub struct PortIo {
    file: File,
}

impl PortIo {
    /// Open `/dev/port` for read/write access.
    ///
    /// Returns `None` if the device doesn't exist or permission is denied.
    pub fn open() -> Option<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/port")
            .ok()?;
        Some(Self { file })
    }

    /// Check if direct I/O port access is available on this system.
    #[allow(dead_code)] // Used in tests
    pub fn is_available() -> bool {
        std::path::Path::new("/dev/port").exists() && unsafe { libc::geteuid() } == 0
    }

    /// Read a single byte from an I/O port.
    pub fn read_byte(&mut self, port: u16) -> std::io::Result<u8> {
        self.file.seek(SeekFrom::Start(port as u64))?;
        let mut buf = [0u8; 1];
        self.file.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Write a single byte to an I/O port.
    pub fn write_byte(&mut self, port: u16, val: u8) -> std::io::Result<()> {
        self.file.seek(SeekFrom::Start(port as u64))?;
        self.file.write_all(&[val])
    }

    /// Write a byte to a port, then read a byte from another port.
    /// Common pattern for Super I/O address/data register pairs.
    pub fn write_read(
        &mut self,
        write_port: u16,
        write_val: u8,
        read_port: u16,
    ) -> std::io::Result<u8> {
        self.write_byte(write_port, write_val)?;
        self.read_byte(read_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available_returns_bool() {
        // Just verify it doesn't panic; actual result depends on environment
        let _ = PortIo::is_available();
    }
}
