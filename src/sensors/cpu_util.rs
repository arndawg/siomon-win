use crate::model::sensor::{SensorCategory, SensorId, SensorReading, SensorUnit};
use std::fs;

pub struct CpuUtilSource {
    prev_jiffies: Vec<CpuJiffies>,
}

#[derive(Clone, Default)]
struct CpuJiffies {
    /// Label from /proc/stat: "cpu" for total, "cpu0", "cpu1", etc.
    name: String,
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl CpuJiffies {
    fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }

    fn idle_total(&self) -> u64 {
        self.idle + self.iowait
    }
}

impl CpuUtilSource {
    pub fn discover() -> Self {
        let prev_jiffies = parse_stat();
        Self { prev_jiffies }
    }

    pub fn poll(&mut self) -> Vec<(SensorId, SensorReading)> {
        let current = parse_stat();
        let mut readings = Vec::new();

        for cur in &current {
            let prev = self.prev_jiffies.iter().find(|p| p.name == cur.name);
            let prev = match prev {
                Some(p) => p,
                None => continue,
            };

            let total_delta = cur.total().saturating_sub(prev.total());
            let idle_delta = cur.idle_total().saturating_sub(prev.idle_total());

            let utilization = if total_delta > 0 {
                100.0 * (1.0 - (idle_delta as f64 / total_delta as f64))
            } else {
                0.0
            };

            let (sensor_name, label) = if cur.name == "cpu" {
                ("total".to_string(), "Total CPU Usage".to_string())
            } else {
                // "cpu0" -> index 0
                let idx_str = &cur.name["cpu".len()..];
                (cur.name.clone(), format!("Core {idx_str} Usage"))
            };

            let id = SensorId {
                source: "cpu".into(),
                chip: "utilization".into(),
                sensor: sensor_name,
            };
            let reading = SensorReading::new(
                label,
                utilization,
                SensorUnit::Percent,
                SensorCategory::Utilization,
            );
            readings.push((id, reading));
        }

        self.prev_jiffies = current;
        readings
    }
}

impl crate::sensors::SensorSource for CpuUtilSource {
    fn name(&self) -> &str {
        "cpu_util"
    }

    fn poll(&mut self) -> Vec<(SensorId, SensorReading)> {
        CpuUtilSource::poll(self)
    }
}

fn parse_stat() -> Vec<CpuJiffies> {
    let Ok(content) = fs::read_to_string("/proc/stat") else {
        return Vec::new();
    };
    let mut result = Vec::new();

    for line in content.lines() {
        if !line.starts_with("cpu") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let name = match parts.next() {
            Some(n) => n,
            None => continue,
        };
        // Must start with "cpu" and either be exactly "cpu" or "cpu" followed by digits
        if name != "cpu"
            && !name
                .strip_prefix("cpu")
                .is_some_and(|s| s.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }

        let fields: Vec<u64> = parts.filter_map(|s| s.parse().ok()).collect();
        if fields.len() < 8 {
            continue;
        }

        result.push(CpuJiffies {
            name: name.to_string(),
            user: fields[0],
            nice: fields[1],
            system: fields[2],
            idle: fields[3],
            iowait: fields[4],
            irq: fields[5],
            softirq: fields[6],
            steal: fields[7],
        });
    }

    result
}
