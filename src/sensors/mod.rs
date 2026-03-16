#[cfg(unix)]
pub mod aer;
pub mod alerts;
pub mod cpu_freq;
pub mod cpu_util;
pub mod disk_activity;
#[cfg(unix)]
pub mod edac;
pub mod gpu_sensors;
#[cfg(unix)]
pub mod hsmp;
#[cfg(unix)]
pub mod hwmon;
#[cfg(unix)]
pub mod i2c;
#[cfg(unix)]
pub mod ipmi;
#[cfg(unix)]
pub mod mce;
pub mod network_stats;
pub mod poller;
#[cfg(unix)]
pub mod rapl;
#[cfg(unix)]
pub mod superio;
#[cfg(windows)]
pub mod ipmi_win;
#[cfg(windows)]
pub mod whea;

use crate::model::sensor::{SensorId, SensorReading};

/// Trait for real-time sensor polling sources.
///
/// Each implementor discovers hardware during construction (not part of the
/// trait, since discovery parameters vary per source) and then polls
/// repeatedly via `poll()`.
pub trait SensorSource: Send {
    /// Human-readable name for logging and timing stats (e.g., "hwmon", "ipmi").
    fn name(&self) -> &str;

    /// Read current sensor values. Returns an empty Vec if the source has
    /// no readings (hardware unavailable, no sensors discovered, etc.).
    fn poll(&mut self) -> Vec<(SensorId, SensorReading)>;
}
