use crate::model::battery::BatteryInfo;
#[cfg(unix)]
use crate::model::battery::{BatteryChemistry, BatteryStatus};
#[cfg(unix)]
use crate::platform::sysfs;
#[cfg(unix)]
use std::path::Path;

#[cfg(not(unix))]
pub fn collect() -> Vec<BatteryInfo> {
    vec![] // Battery detection requires additional Windows API features
}

#[cfg(unix)]
pub fn collect() -> Vec<BatteryInfo> {
    let mut batteries = Vec::new();

    for entry in sysfs::glob_paths("/sys/class/power_supply/*") {
        let name = match entry.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        // Only process Battery-type power supplies
        let supply_type = sysfs::read_string_optional(&entry.join("type")).unwrap_or_default();
        if supply_type != "Battery" {
            continue;
        }

        if let Some(battery) = collect_battery(&name, &entry) {
            batteries.push(battery);
        }
    }

    batteries.sort_by(|a, b| a.name.cmp(&b.name));
    batteries
}

#[cfg(unix)]
pub struct BatteryCollector;

#[cfg(unix)]
impl crate::collectors::Collector for BatteryCollector {
    fn name(&self) -> &str {
        "battery"
    }

    fn collect_into(&self, info: &mut crate::model::system::SystemInfo) {
        info.batteries = collect();
    }
}

#[cfg(unix)]
fn collect_battery(name: &str, path: &Path) -> Option<BatteryInfo> {
    let manufacturer = sysfs::read_string_optional(&path.join("manufacturer"));
    let model_name = sysfs::read_string_optional(&path.join("model_name"));

    let chemistry = sysfs::read_string_optional(&path.join("technology"))
        .map(|s| classify_chemistry(&s))
        .unwrap_or(BatteryChemistry::Unknown("unknown".into()));

    let status = sysfs::read_string_optional(&path.join("status"))
        .map(|s| classify_status(&s))
        .unwrap_or(BatteryStatus::Unknown);

    let design_capacity_uwh = sysfs::read_u64_optional(&path.join("energy_full_design"));
    let full_charge_capacity_uwh = sysfs::read_u64_optional(&path.join("energy_full"));
    let remaining_capacity_uwh = sysfs::read_u64_optional(&path.join("energy_now"));
    let voltage_now_uv = sysfs::read_u64_optional(&path.join("voltage_now"));
    let power_now_uw = sysfs::read_u64_optional(&path.join("power_now"))
        .or_else(|| compute_power_from_current(path));

    let capacity_percent = sysfs::read_u64_optional(&path.join("capacity")).map(|v| v as u8);
    let cycle_count = sysfs::read_u32_optional(&path.join("cycle_count"));

    let wear_percent = match (full_charge_capacity_uwh, design_capacity_uwh) {
        (Some(full), Some(design)) if design > 0 => Some(1.0 - (full as f64 / design as f64)),
        _ => None,
    };

    Some(BatteryInfo {
        name: name.to_string(),
        manufacturer,
        model_name,
        chemistry,
        status,
        design_capacity_uwh,
        full_charge_capacity_uwh,
        remaining_capacity_uwh,
        voltage_now_uv,
        power_now_uw,
        capacity_percent,
        cycle_count,
        wear_percent,
    })
}

#[cfg(unix)]
fn compute_power_from_current(path: &Path) -> Option<u64> {
    let current_ua = sysfs::read_u64_optional(&path.join("current_now"))?;
    let voltage_uv = sysfs::read_u64_optional(&path.join("voltage_now"))?;
    // P = I * V; current_now is in uA, voltage_now is in uV
    // power in uW = (current_uA * voltage_uV) / 1_000_000
    Some(current_ua * voltage_uv / 1_000_000)
}

#[cfg(unix)]
fn classify_chemistry(technology: &str) -> BatteryChemistry {
    match technology {
        "Li-ion" => BatteryChemistry::LithiumIon,
        "Li-poly" => BatteryChemistry::LithiumPolymer,
        "NiMH" => BatteryChemistry::NickelMetalHydride,
        "NiCd" => BatteryChemistry::NickelCadmium,
        other => BatteryChemistry::Unknown(other.to_string()),
    }
}

#[cfg(unix)]
fn classify_status(status: &str) -> BatteryStatus {
    match status {
        "Charging" => BatteryStatus::Charging,
        "Discharging" => BatteryStatus::Discharging,
        "Full" => BatteryStatus::Full,
        "Not charging" => BatteryStatus::NotCharging,
        _ => BatteryStatus::Unknown,
    }
}
