use crate::model::audio::{AudioBusType, AudioDevice};

// ── Unix (Linux /proc/asound) implementation ───────────────────────────────

#[cfg(unix)]
pub fn collect() -> Vec<AudioDevice> {
    let mut devices = Vec::new();

    let Ok(content) = std::fs::read_to_string("/proc/asound/cards") else {
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

#[cfg(unix)]
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

#[cfg(unix)]
fn read_codec(card_index: u32) -> Option<String> {
    let codec_path = format!("/proc/asound/card{}/codec#0", card_index);
    let content = std::fs::read_to_string(&codec_path).ok()?;
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

#[cfg(unix)]
fn read_pci_address(card_index: u32) -> Option<String> {
    use std::path::Path;
    let device_link = format!("/sys/class/sound/card{}/device", card_index);
    let path = Path::new(&device_link);
    path.canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
}

// ── Windows (WMI via PowerShell) implementation ────────────────────────────

#[cfg(windows)]
pub fn collect() -> Vec<AudioDevice> {
    collect_via_powershell().unwrap_or_default()
}

/// Query Win32_SoundDevice for audio devices.
#[cfg(windows)]
fn collect_via_powershell() -> Option<Vec<AudioDevice>> {
    let ps_script = r#"
$devs = Get-CimInstance Win32_SoundDevice
$result = @()
$idx = 0
foreach ($d in $devs) {
    $obj = [ordered]@{
        Name         = $d.Name
        Manufacturer = $d.Manufacturer
        DeviceID     = $d.DeviceID
        Status       = $d.Status
        Index        = $idx
    }
    $result += $obj
    $idx++
}
$result | ConvertTo-Json -Compress
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .output()
        .ok()?;

    if !output.status.success() {
        log::debug!(
            "Audio PowerShell query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();
    if stdout.is_empty() {
        return Some(Vec::new());
    }

    let json_str = if stdout.starts_with('[') {
        stdout.to_string()
    } else {
        format!("[{}]", stdout)
    };

    let raw: Vec<WmiAudioRow> = serde_json::from_str(&json_str).ok()?;

    let mut devices: Vec<AudioDevice> = raw.iter().enumerate().map(|(i, r)| {
        wmi_row_to_audio(r, i as u32)
    }).collect();
    devices.sort_by_key(|d| d.card_index);
    Some(devices)
}

#[cfg(windows)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct WmiAudioRow {
    name: Option<String>,
    manufacturer: Option<String>,
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
    status: Option<String>,
    index: Option<u32>,
}

#[cfg(windows)]
fn wmi_row_to_audio(row: &WmiAudioRow, idx: u32) -> AudioDevice {
    let name = row.name.clone().unwrap_or_default();
    let device_id = row.device_id.as_deref().unwrap_or("");

    // Classify bus type from the DeviceID prefix
    let bus_type = if device_id.starts_with("HDAUDIO") {
        AudioBusType::HdAudio
    } else if device_id.starts_with("USB") || device_id.contains("VID_") {
        AudioBusType::Usb
    } else if device_id.starts_with("PCI") {
        // Non-HDA PCI audio (AC97, etc.)
        AudioBusType::Ac97
    } else if device_id.contains("ROOT") || device_id.contains("VIRTUAL") || device_id.contains("SW") {
        AudioBusType::Virtual
    } else {
        AudioBusType::Unknown(device_id.split('\\').next().unwrap_or("unknown").to_string())
    };

    // Extract PCI address from DeviceID if it contains PCI-like info
    let pci_bus_address = extract_pci_address(device_id);

    // Use manufacturer as driver hint
    let driver = row.manufacturer.clone().unwrap_or_else(|| "Unknown".to_string());

    AudioDevice {
        card_index: row.index.unwrap_or(idx),
        card_id: name.clone(),
        card_long_name: name,
        driver,
        bus_type,
        codec: None, // Not available from WMI
        pci_bus_address,
    }
}

/// Try to extract a PCI bus address from a Windows HDAUDIO or PCI DeviceID.
/// Example: `HDAUDIO\FUNC_01&VEN_10EC&DEV_0892&SUBSYS_10438723&REV_1003\5&2F4E3A01&0&0001`
#[cfg(windows)]
fn extract_pci_address(device_id: &str) -> Option<String> {
    // If the DeviceID contains VEN_ and DEV_, extract them for display
    let ven = {
        let marker = "VEN_";
        let start = device_id.find(marker)? + marker.len();
        let hex: String = device_id[start..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        hex
    };
    let dev = {
        let marker = "DEV_";
        let start = device_id.find(marker)? + marker.len();
        let hex: String = device_id[start..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        hex
    };
    Some(format!("VEN_{}&DEV_{}", ven, dev))
}

// ── Shared ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn classify_bus_type(driver: &str) -> AudioBusType {
    match driver {
        "HDA-Intel" => AudioBusType::HdAudio,
        "USB-Audio" => AudioBusType::Usb,
        "AC97" => AudioBusType::Ac97,
        "Dummy" | "Loopback" => AudioBusType::Virtual,
        other => AudioBusType::Unknown(other.to_string()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_bus_type() {
        assert_eq!(classify_bus_type("HDA-Intel"), AudioBusType::HdAudio);
        assert_eq!(classify_bus_type("USB-Audio"), AudioBusType::Usb);
        assert_eq!(
            classify_bus_type("SomeOther"),
            AudioBusType::Unknown("SomeOther".to_string())
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_extract_pci_address() {
        let did = r"HDAUDIO\FUNC_01&VEN_10EC&DEV_0892&SUBSYS_10438723&REV_1003\5&2F4E3A01&0&0001";
        assert_eq!(
            extract_pci_address(did),
            Some("VEN_10EC&DEV_0892".to_string())
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_wmi_row_to_audio() {
        let row = WmiAudioRow {
            name: Some("Realtek High Definition Audio".to_string()),
            manufacturer: Some("Realtek".to_string()),
            device_id: Some(
                r"HDAUDIO\FUNC_01&VEN_10EC&DEV_0892&SUBSYS_10438723&REV_1003\5&1234&0&0001"
                    .to_string(),
            ),
            status: Some("OK".to_string()),
            index: Some(0),
        };

        let dev = wmi_row_to_audio(&row, 0);
        assert_eq!(dev.bus_type, AudioBusType::HdAudio);
        assert_eq!(dev.card_id, "Realtek High Definition Audio");
        assert_eq!(dev.driver, "Realtek");
    }
}
