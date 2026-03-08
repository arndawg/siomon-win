use crate::platform::sysfs;

/// A discovered I2C bus with its adapter classification.
pub struct I2cBus {
    pub bus_num: u32,
    pub adapter_type: I2cAdapterType,
}

/// Classification of the I2C adapter by its kernel driver.
///
/// Only SMBus host adapters (Piix4, I801) are suitable for scanning
/// SPD and sensor devices. GPU and platform I2C buses are skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I2cAdapterType {
    /// AMD FCH (piix4_smbus)
    Piix4Smbus,
    /// Intel PCH (i2c-i801)
    I801,
    /// NVIDIA GPU I2C — skip for sensor scanning
    Nvidia,
    /// DesignWare platform I2C — skip for sensor scanning
    DesignWare,
    /// Unrecognized adapter
    Other(String),
}

impl I2cAdapterType {
    /// Whether this adapter type is an SMBus host suitable for device scanning.
    pub fn is_smbus(&self) -> bool {
        matches!(self, Self::Piix4Smbus | Self::I801)
    }
}

/// Classify an adapter name string from sysfs into an adapter type.
fn classify_adapter(name: &str) -> I2cAdapterType {
    let lower = name.to_lowercase();
    if lower.contains("piix4") {
        I2cAdapterType::Piix4Smbus
    } else if lower.contains("i801") {
        I2cAdapterType::I801
    } else if lower.contains("nvidia") || lower.contains("nouveau") {
        I2cAdapterType::Nvidia
    } else if lower.contains("designware") || lower.contains("synopsys") {
        I2cAdapterType::DesignWare
    } else {
        I2cAdapterType::Other(name.to_string())
    }
}

/// Enumerate all I2C buses visible in sysfs.
///
/// Reads `/sys/bus/i2c/devices/i2c-*/name` to discover adapters and
/// classifies each by its driver name.
pub fn enumerate_buses() -> Vec<I2cBus> {
    let mut buses = Vec::new();

    for path in sysfs::glob_paths("/sys/bus/i2c/devices/i2c-*/name") {
        // Extract bus number from path: ".../i2c-7/name" -> 7
        let parent = match path.parent() {
            Some(p) => p,
            None => continue,
        };
        let dir_name = match parent.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let bus_num: u32 = match dir_name.strip_prefix("i2c-").and_then(|s| s.parse().ok()) {
            Some(n) => n,
            None => continue,
        };

        let name = match sysfs::read_string_optional(&path) {
            Some(n) => n,
            None => continue,
        };

        let adapter_type = classify_adapter(&name);

        buses.push(I2cBus {
            bus_num,
            adapter_type,
        });
    }

    buses.sort_by_key(|b| b.bus_num);
    buses
}

/// Enumerate only SMBus host adapters suitable for device scanning.
///
/// This is a convenience wrapper around [`enumerate_buses`] that filters
/// out GPU, platform, and other non-SMBus adapters.
pub fn enumerate_smbus_adapters() -> Vec<I2cBus> {
    enumerate_buses()
        .into_iter()
        .filter(|b| b.adapter_type.is_smbus())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_piix4() {
        assert_eq!(
            classify_adapter("SMBus PIIX4 adapter port 0 at 0b00"),
            I2cAdapterType::Piix4Smbus
        );
    }

    #[test]
    fn classify_i801() {
        assert_eq!(
            classify_adapter("SMBus I801 adapter at efa0"),
            I2cAdapterType::I801
        );
    }

    #[test]
    fn classify_nvidia() {
        assert_eq!(
            classify_adapter("NVIDIA i2c adapter 1 at 1:00.0"),
            I2cAdapterType::Nvidia
        );
    }

    #[test]
    fn classify_designware() {
        assert_eq!(
            classify_adapter("Synopsys DesignWare I2C adapter"),
            I2cAdapterType::DesignWare
        );
    }

    #[test]
    fn classify_other() {
        let t = classify_adapter("Some Unknown Adapter");
        assert!(matches!(t, I2cAdapterType::Other(_)));
    }

    #[test]
    fn smbus_check() {
        assert!(I2cAdapterType::Piix4Smbus.is_smbus());
        assert!(I2cAdapterType::I801.is_smbus());
        assert!(!I2cAdapterType::Nvidia.is_smbus());
        assert!(!I2cAdapterType::DesignWare.is_smbus());
        assert!(!I2cAdapterType::Other("x".into()).is_smbus());
    }
}
