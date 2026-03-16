#[cfg(unix)]
pub mod msr;
#[cfg(unix)]
pub mod nvme_ioctl;
#[cfg(feature = "nvidia")]
pub mod nvml;
#[cfg(unix)]
pub mod port_io;
pub mod procfs;
#[cfg(unix)]
pub mod sata_ioctl;
#[cfg(unix)]
pub mod sinfo_io;
pub mod sysfs;
