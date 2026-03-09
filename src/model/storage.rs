use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub device_name: String,
    pub sysfs_path: String,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub capacity_bytes: u64,
    pub interface: StorageInterface,
    pub rotational: bool,
    pub logical_sector_size: u32,
    pub physical_sector_size: u32,
    pub nvme: Option<NvmeDetails>,
    pub smart: Option<SmartData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum StorageInterface {
    NVMe,
    SATA,
    SAS,
    USB,
    MMC,
    VirtIO,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeDetails {
    pub controller_id: u32,
    pub nvme_version: Option<String>,
    pub transport: String,
    pub namespace_count: u32,
    pub controller_type: Option<String>,
    pub queue_count: Option<u32>,
    pub subsystem_nqn: Option<String>,
    pub smart: Option<SmartData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartData {
    pub temperature_celsius: i32,
    pub available_spare_pct: u8,
    pub available_spare_threshold_pct: u8,
    pub percentage_used: u8,
    pub data_units_read: u128,
    pub data_units_written: u128,
    pub host_read_commands: u128,
    pub host_write_commands: u128,
    pub controller_busy_time_minutes: u128,
    pub power_cycles: u128,
    pub power_on_hours: u128,
    pub unsafe_shutdowns: u128,
    pub media_errors: u128,
    pub num_error_log_entries: u128,
    pub warning_composite_temp_time_minutes: u32,
    pub critical_composite_temp_time_minutes: u32,
    pub critical_warning: u8,
    pub total_bytes_read: u128,
    pub total_bytes_written: u128,
}
