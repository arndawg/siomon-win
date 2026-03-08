use crate::model::sensor::{SensorCategory, SensorId, SensorReading, SensorUnit};
use crate::platform::sysfs;
use std::path::PathBuf;

pub struct CpuFreqSource {
    cpus: Vec<CpuFreqEntry>,
}

struct CpuFreqEntry {
    index: u32,
    freq_path: PathBuf,
}

impl CpuFreqSource {
    pub fn discover() -> Self {
        let mut cpus = Vec::new();

        for path in sysfs::glob_paths("/sys/devices/system/cpu/cpu[0-9]*/cpufreq/scaling_cur_freq")
        {
            // Extract CPU index from path: .../cpu{N}/cpufreq/...
            let cpu_dir = match path.parent().and_then(|p| p.parent()) {
                Some(d) => d,
                None => continue,
            };
            let dir_name = match cpu_dir.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            let idx: u32 = match dir_name.strip_prefix("cpu").and_then(|s| s.parse().ok()) {
                Some(i) => i,
                None => continue,
            };

            cpus.push(CpuFreqEntry {
                index: idx,
                freq_path: path,
            });
        }

        cpus.sort_by_key(|e| e.index);

        Self { cpus }
    }

    pub fn poll(&self) -> Vec<(SensorId, SensorReading)> {
        let mut readings = Vec::new();

        for entry in &self.cpus {
            let Some(khz) = sysfs::read_u64_optional(&entry.freq_path) else {
                continue;
            };
            let mhz = khz as f64 / 1000.0;

            let id = SensorId {
                source: "cpu".into(),
                chip: "cpufreq".into(),
                sensor: format!("cpu{}", entry.index),
            };
            let label = format!("Core {} Frequency", entry.index);
            let reading =
                SensorReading::new(label, mhz, SensorUnit::Mhz, SensorCategory::Frequency);
            readings.push((id, reading));
        }

        readings
    }
}

impl crate::sensors::SensorSource for CpuFreqSource {
    fn name(&self) -> &str {
        "cpufreq"
    }

    fn poll(&mut self) -> Vec<(SensorId, SensorReading)> {
        CpuFreqSource::poll(self)
    }
}
