use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::audio::AudioDevice;
use super::battery::BatteryInfo;
use super::cpu::CpuInfo;
use super::gpu::GpuInfo;
use super::memory::MemoryInfo;
use super::motherboard::MotherboardInfo;
use super::network::NetworkAdapter;
use super::pci::PciDevice;
use super::sensor::SensorSnapshot;
use super::storage::StorageDevice;
use super::usb::UsbDevice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub hostname: String,
    pub kernel_version: String,
    pub os_name: Option<String>,
    pub cpus: Vec<CpuInfo>,
    pub memory: MemoryInfo,
    pub motherboard: MotherboardInfo,
    pub gpus: Vec<GpuInfo>,
    pub storage: Vec<StorageDevice>,
    pub network: Vec<NetworkAdapter>,
    pub audio: Vec<AudioDevice>,
    pub usb_devices: Vec<UsbDevice>,
    pub pci_devices: Vec<PciDevice>,
    pub batteries: Vec<BatteryInfo>,
    pub sensors: Option<SensorSnapshot>,
}
