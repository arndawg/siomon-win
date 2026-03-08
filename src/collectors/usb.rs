use crate::model::usb::{UsbDevice, UsbSpeed};
use crate::platform::sysfs;
use std::path::Path;

pub fn collect() -> Vec<UsbDevice> {
    let mut devices = Vec::new();

    for entry in sysfs::glob_paths("/sys/bus/usb/devices/*") {
        let name = match entry.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip interfaces (entries containing ":")
        if name.contains(':') {
            continue;
        }

        if let Some(device) = collect_device(&name, &entry) {
            devices.push(device);
        }
    }

    devices.sort_by(|a, b| {
        a.bus
            .cmp(&b.bus)
            .then_with(|| a.port_path.cmp(&b.port_path))
    });
    devices
}

pub struct UsbCollector;

impl crate::collectors::Collector for UsbCollector {
    fn name(&self) -> &str {
        "usb"
    }

    fn collect_into(&self, info: &mut crate::model::system::SystemInfo) {
        info.usb_devices = collect();
    }
}

fn collect_device(name: &str, path: &Path) -> Option<UsbDevice> {
    let vendor_id = read_hex_u16(path, "idVendor")?;
    let product_id = read_hex_u16(path, "idProduct")?;

    let bus = sysfs::read_u64_optional(&path.join("busnum"))? as u8;
    let devnum = sysfs::read_u64_optional(&path.join("devnum"))? as u16;
    let port_path =
        sysfs::read_string_optional(&path.join("devpath")).unwrap_or_else(|| "0".into());

    let manufacturer = sysfs::read_string_optional(&path.join("manufacturer"));
    let product = sysfs::read_string_optional(&path.join("product"));
    let serial_number = sysfs::read_string_optional(&path.join("serial"));
    let usb_version = sysfs::read_string_optional(&path.join("version")).map(|s| s.trim().into());

    let device_class = sysfs::read_string_optional(&path.join("bDeviceClass"))
        .and_then(|s| u8::from_str_radix(&s, 16).ok())
        .unwrap_or(0);

    let speed = sysfs::read_string_optional(&path.join("speed"))
        .map(|s| classify_speed(&s))
        .unwrap_or(UsbSpeed::Unknown("unknown".into()));

    let max_power_ma =
        sysfs::read_string_optional(&path.join("bMaxPower")).and_then(|s| parse_max_power(&s));

    Some(UsbDevice {
        bus,
        port_path,
        devnum,
        vendor_id,
        product_id,
        manufacturer,
        product,
        serial_number,
        usb_version,
        device_class,
        speed,
        max_power_ma,
        sysfs_id: name.to_string(),
    })
}

fn read_hex_u16(path: &Path, attr: &str) -> Option<u16> {
    sysfs::read_string_optional(&path.join(attr)).and_then(|s| u16::from_str_radix(&s, 16).ok())
}

fn classify_speed(speed: &str) -> UsbSpeed {
    match speed {
        "1.5" => UsbSpeed::Low,
        "12" => UsbSpeed::Full,
        "480" => UsbSpeed::High,
        "5000" => UsbSpeed::Super,
        "10000" => UsbSpeed::SuperPlus,
        "20000" => UsbSpeed::SuperPlus2x2,
        other => UsbSpeed::Unknown(other.to_string()),
    }
}

fn parse_max_power(s: &str) -> Option<u32> {
    // Formats: "500mA" or "0mA"
    s.strip_suffix("mA")
        .and_then(|v| v.trim().parse::<u32>().ok())
}
