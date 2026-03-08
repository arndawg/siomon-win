use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_free_bytes: u64,
    pub max_capacity_bytes: Option<u64>,
    pub total_slots: Option<u32>,
    pub populated_slots: Option<u32>,
    pub dimms: Vec<DimmInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimmInfo {
    pub locator: String,
    pub bank_locator: Option<String>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub serial_number: Option<String>,
    pub size_bytes: u64,
    pub memory_type: MemoryType,
    pub form_factor: String,
    pub type_detail: Option<String>,
    pub configured_speed_mts: Option<u32>,
    pub max_speed_mts: Option<u32>,
    pub configured_voltage_mv: Option<u32>,
    pub data_width_bits: Option<u16>,
    pub total_width_bits: Option<u16>,
    pub ecc: bool,
    pub rank: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryType {
    DDR3,
    DDR4,
    DDR5,
    LPDDR4,
    LPDDR5,
    LPDDR5X,
    Unknown(String),
}
