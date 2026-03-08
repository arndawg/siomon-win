use crate::model::sensor::{SensorCategory, SensorId, SensorReading, SensorUnit};
use crate::platform::sysfs;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

pub struct NetworkStatsSource {
    prev_stats: HashMap<String, NetStat>,
    prev_time: Instant,
}

#[derive(Clone)]
struct NetStat {
    rx_bytes: u64,
    tx_bytes: u64,
}

impl NetworkStatsSource {
    pub fn discover() -> Self {
        let mut prev_stats = HashMap::new();

        for dir in sysfs::glob_paths("/sys/class/net/*") {
            let iface = match dir.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if !is_physical_interface(&dir, &iface) {
                continue;
            }

            if let Some(stat) = read_net_stat(&iface) {
                prev_stats.insert(iface, stat);
            }
        }

        Self {
            prev_stats,
            prev_time: Instant::now(),
        }
    }

    pub fn poll(&mut self) -> Vec<(SensorId, SensorReading)> {
        let now = Instant::now();
        let elapsed_secs = now.duration_since(self.prev_time).as_secs_f64();
        let mut readings = Vec::new();

        if elapsed_secs <= 0.0 {
            self.prev_time = now;
            return readings;
        }

        let mut current_stats = HashMap::new();

        for iface in self.prev_stats.keys() {
            let Some(cur) = read_net_stat(iface) else {
                continue;
            };

            if let Some(prev) = self.prev_stats.get(iface) {
                let rx_delta = cur.rx_bytes.saturating_sub(prev.rx_bytes);
                let tx_delta = cur.tx_bytes.saturating_sub(prev.tx_bytes);

                let rx_mbps = (rx_delta as f64) / (1_048_576.0 * elapsed_secs);
                let tx_mbps = (tx_delta as f64) / (1_048_576.0 * elapsed_secs);

                let rx_id = SensorId {
                    source: "net".into(),
                    chip: iface.clone(),
                    sensor: "rx_mbps".into(),
                };
                let rx_label = format!("{iface} RX");
                readings.push((
                    rx_id,
                    SensorReading::new(
                        rx_label,
                        rx_mbps,
                        SensorUnit::MegabytesPerSec,
                        SensorCategory::Throughput,
                    ),
                ));

                let tx_id = SensorId {
                    source: "net".into(),
                    chip: iface.clone(),
                    sensor: "tx_mbps".into(),
                };
                let tx_label = format!("{iface} TX");
                readings.push((
                    tx_id,
                    SensorReading::new(
                        tx_label,
                        tx_mbps,
                        SensorUnit::MegabytesPerSec,
                        SensorCategory::Throughput,
                    ),
                ));
            }

            current_stats.insert(iface.clone(), cur);
        }

        self.prev_stats = current_stats;
        self.prev_time = now;
        readings
    }
}

impl crate::sensors::SensorSource for NetworkStatsSource {
    fn name(&self) -> &str {
        "network"
    }

    fn poll(&mut self) -> Vec<(SensorId, SensorReading)> {
        NetworkStatsSource::poll(self)
    }
}

fn read_net_stat(iface: &str) -> Option<NetStat> {
    let base = Path::new("/sys/class/net").join(iface).join("statistics");
    let rx_bytes = sysfs::read_u64_optional(&base.join("rx_bytes"))?;
    let tx_bytes = sysfs::read_u64_optional(&base.join("tx_bytes"))?;
    Some(NetStat { rx_bytes, tx_bytes })
}

fn is_physical_interface(dir: &Path, iface: &str) -> bool {
    // Skip loopback
    if iface == "lo" {
        return false;
    }

    // Virtual interfaces don't have a "device" symlink in sysfs
    // Physical NICs (PCI, USB) have /sys/class/net/{iface}/device -> ../../...
    dir.join("device").exists()
}
