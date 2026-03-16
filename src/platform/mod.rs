#[cfg(unix)]
pub mod msr;
#[cfg(windows)]
pub mod msr_win;
#[cfg(unix)]
pub mod nvme_ioctl;
#[cfg(feature = "nvidia")]
pub mod nvml;
#[cfg(all(windows, feature = "nvidia"))]
pub mod adl;
#[cfg(windows)]
pub mod nvme_win;
#[cfg(unix)]
pub mod port_io;
#[cfg(windows)]
pub mod port_io_win;
pub mod procfs;
#[cfg(unix)]
pub mod sata_ioctl;
#[cfg(windows)]
pub mod sata_win;
#[cfg(unix)]
pub mod sinfo_io;
pub mod sysfs;
#[cfg(windows)]
pub mod winring0;
