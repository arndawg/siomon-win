use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub version: Option<String>,
    pub serial_number: Option<String>,
    pub system_vendor: Option<String>,
    pub system_product: Option<String>,
    pub system_family: Option<String>,
    pub system_sku: Option<String>,
    pub system_uuid: Option<String>,
    pub chassis_type: Option<String>,
    pub bios: BiosInfo,
    pub chipset: Option<String>,
    pub me_version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BiosInfo {
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub date: Option<String>,
    pub release: Option<String>,
    pub uefi_boot: bool,
    pub secure_boot: Option<bool>,
}
