use crate::model::audio::{AudioBusType, AudioDevice};
use std::fs;
use std::path::Path;

pub fn collect() -> Vec<AudioDevice> {
    let mut devices = Vec::new();

    let Ok(content) = fs::read_to_string("/proc/asound/cards") else {
        return devices;
    };

    let lines: Vec<&str> = content.lines().collect();
    // Cards file has two lines per card: header line and detail line
    for chunk in lines.chunks(2) {
        if chunk.is_empty() {
            continue;
        }
        if let Some(device) = parse_card(chunk[0]) {
            devices.push(device);
        }
    }

    devices.sort_by_key(|d| d.card_index);
    devices
}

pub struct AudioCollector;

impl crate::collectors::Collector for AudioCollector {
    fn name(&self) -> &str {
        "audio"
    }

    fn collect_into(&self, info: &mut crate::model::system::SystemInfo) {
        info.audio = collect();
    }
}

fn parse_card(header: &str) -> Option<AudioDevice> {
    // Format: " 0 [NVidia         ]: HDA-Intel - HDA NVidia"
    let header = header.trim();

    // Extract card index (leading number)
    let (index_str, rest) = header.split_once('[')?;
    let card_index = index_str.trim().parse::<u32>().ok()?;

    // Extract card_id (bracketed name)
    let (card_id_raw, rest) = rest.split_once(']')?;
    let card_id = card_id_raw.trim().to_string();

    // After "]: " comes "driver - long_name"
    let rest = rest.strip_prefix(": ")?.trim();
    let (driver, card_long_name) = if let Some((drv, name)) = rest.split_once(" - ") {
        (drv.trim().to_string(), name.trim().to_string())
    } else {
        (rest.to_string(), String::new())
    };

    let bus_type = classify_bus_type(&driver);
    let codec = read_codec(card_index);
    let pci_bus_address = read_pci_address(card_index);

    Some(AudioDevice {
        card_index,
        card_id,
        card_long_name,
        driver,
        bus_type,
        codec,
        pci_bus_address,
    })
}

fn classify_bus_type(driver: &str) -> AudioBusType {
    match driver {
        "HDA-Intel" => AudioBusType::HdAudio,
        "USB-Audio" => AudioBusType::Usb,
        "AC97" => AudioBusType::Ac97,
        "Dummy" | "Loopback" => AudioBusType::Virtual,
        other => AudioBusType::Unknown(other.to_string()),
    }
}

fn read_codec(card_index: u32) -> Option<String> {
    let codec_path = format!("/proc/asound/card{}/codec#0", card_index);
    let content = fs::read_to_string(&codec_path).ok()?;
    for line in content.lines() {
        if let Some(codec_value) = line.strip_prefix("Codec:") {
            let codec = codec_value.trim();
            if !codec.is_empty() {
                return Some(codec.to_string());
            }
        }
    }
    None
}

fn read_pci_address(card_index: u32) -> Option<String> {
    let device_link = format!("/sys/class/sound/card{}/device", card_index);
    let path = Path::new(&device_link);
    path.canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
}
