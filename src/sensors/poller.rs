use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::model::sensor::{SensorId, SensorReading};
use crate::sensors::{
    cpu_freq, cpu_util, disk_activity, gpu_sensors, hwmon, network_stats, rapl, superio,
    SensorSource,
};

pub type SensorState = Arc<RwLock<HashMap<SensorId, SensorReading>>>;

pub fn new_state() -> SensorState {
    Arc::new(RwLock::new(HashMap::new()))
}

#[derive(Debug, Clone, Default)]
pub struct PollStats {
    pub cycle_duration_ms: u64,
    pub source_durations: HashMap<String, u64>, // name -> ms
}

pub type PollStatsState = Arc<RwLock<PollStats>>;

pub fn new_poll_stats() -> PollStatsState {
    Arc::new(RwLock::new(PollStats::default()))
}

pub struct Poller {
    state: SensorState,
    poll_stats: PollStatsState,
    interval: Duration,
    no_nvidia: bool,
    direct_io: bool,
    label_overrides: HashMap<String, String>,
}

impl Poller {
    pub fn new(
        state: SensorState,
        poll_stats: PollStatsState,
        interval_ms: u64,
        no_nvidia: bool,
        direct_io: bool,
        label_overrides: HashMap<String, String>,
    ) -> Self {
        Self {
            state,
            poll_stats,
            interval: Duration::from_millis(interval_ms),
            no_nvidia,
            direct_io,
            label_overrides,
        }
    }

    /// Run the polling loop in a background thread. Returns a handle to stop it.
    pub fn spawn(self) -> PollerHandle {
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_clone = stop.clone();

        let handle = thread::spawn(move || {
            self.run(stop_clone);
        });

        PollerHandle {
            stop,
            _handle: handle,
        }
    }

    fn run(self, stop: Arc<std::sync::atomic::AtomicBool>) {
        let mut sources =
            discover_all_sources(self.no_nvidia, self.direct_io, &self.label_overrides);

        log::info!("Sensor poller started: {} sources", sources.len());

        let mut durations: HashMap<String, u64> = HashMap::new();
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            let cycle_start = Instant::now();
            let mut new_readings: Vec<(SensorId, SensorReading)> = Vec::new();
            durations.clear();

            for source in &mut sources {
                let t = Instant::now();
                new_readings.extend(source.poll());
                *durations.entry(source.name().to_string()).or_default() +=
                    t.elapsed().as_millis() as u64;
            }

            let cycle_ms = cycle_start.elapsed().as_millis() as u64;

            // Log warning for slow poll cycles
            if cycle_ms > 500 {
                let slow: Vec<String> = durations
                    .iter()
                    .filter(|&(_, &ms)| ms > 100)
                    .map(|(name, ms)| format!("{name}: {ms}ms"))
                    .collect();
                log::warn!(
                    "Slow poll cycle: {}ms [{}]",
                    cycle_ms,
                    if slow.is_empty() {
                        "no single source >100ms".into()
                    } else {
                        slow.join(", ")
                    }
                );
            }

            // Update shared state
            if let Ok(mut state) = self.state.write() {
                for (id, new_reading) in new_readings {
                    if let Some(existing) = state.get_mut(&id) {
                        existing.update(new_reading.current);
                    } else {
                        state.insert(id, new_reading);
                    }
                }
            }

            // Update poll stats
            if let Ok(mut stats) = self.poll_stats.write() {
                stats.cycle_duration_ms = cycle_ms;
                stats.source_durations.clone_from(&durations);
            }

            thread::sleep(self.interval);
        }
    }
}

pub struct PollerHandle {
    stop: Arc<std::sync::atomic::AtomicBool>,
    _handle: thread::JoinHandle<()>,
}

impl PollerHandle {
    pub fn stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Drop for PollerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Discover all sensor sources and return them as trait objects.
///
/// Encapsulates per-source construction and logging. Called by both the
/// continuous poller and the one-shot snapshot.
fn discover_all_sources(
    no_nvidia: bool,
    direct_io: bool,
    label_overrides: &HashMap<String, String>,
) -> Vec<Box<dyn SensorSource>> {
    let mut sources: Vec<Box<dyn SensorSource>> = Vec::new();

    let hwmon_src = hwmon::HwmonSource::discover(label_overrides);
    log::info!(
        "hwmon: {} chips, {} sensors",
        hwmon_src.chip_count(),
        hwmon_src.sensor_count()
    );
    sources.push(Box::new(hwmon_src));

    sources.push(Box::new(cpu_freq::CpuFreqSource::discover()));
    sources.push(Box::new(cpu_util::CpuUtilSource::discover()));
    sources.push(Box::new(gpu_sensors::GpuSensorSource::discover(no_nvidia)));
    sources.push(Box::new(rapl::RaplSource::discover()));
    sources.push(Box::new(disk_activity::DiskActivitySource::discover()));
    sources.push(Box::new(network_stats::NetworkStatsSource::discover()));

    // Direct I/O sources (Super I/O, I2C) — only when --direct-io is set
    if direct_io {
        let chips = superio::chip_detect::detect_all();
        let mut nct_count = 0;
        let mut ite_count = 0;
        for chip in chips {
            let nct_s = superio::nct67xx::Nct67xxSource::new(chip.clone());
            if nct_s.is_supported() {
                nct_count += 1;
                sources.push(Box::new(nct_s));
                continue;
            }
            let ite_s = superio::ite87xx::Ite87xxSource::new(chip);
            if ite_s.is_supported() {
                ite_count += 1;
                sources.push(Box::new(ite_s));
            }
        }
        if nct_count > 0 || ite_count > 0 {
            log::info!(
                "Super I/O: {} nct chips, {} ite chips",
                nct_count,
                ite_count
            );
        }

        let buses = crate::sensors::i2c::bus_scan::enumerate_smbus_adapters();
        sources.push(Box::new(
            crate::sensors::i2c::spd5118::Spd5118Source::discover(&buses),
        ));
        sources.push(Box::new(crate::sensors::i2c::pmbus::PmbusSource::discover(
            &buses,
        )));
        log::info!("I2C: enabled ({} buses)", buses.len());
    }

    // HSMP — always try (don't require --direct-io)
    let hsmp_src = super::hsmp::HsmpSource::discover();
    log::info!(
        "HSMP: {}",
        if hsmp_src.is_available() { "yes" } else { "no" }
    );
    sources.push(Box::new(hsmp_src));

    // IPMI — native ioctl via ipmi-rs, fast enough for the main loop
    let ipmi_src = super::ipmi::IpmiSource::discover();
    log::info!(
        "IPMI: {}",
        if ipmi_src.is_available() { "yes" } else { "no" }
    );
    sources.push(Box::new(ipmi_src));

    sources
}

/// Take a one-shot snapshot of all sensors (single poll cycle).
pub fn snapshot(
    no_nvidia: bool,
    direct_io: bool,
    label_overrides: &HashMap<String, String>,
) -> HashMap<SensorId, SensorReading> {
    let mut sources = discover_all_sources(no_nvidia, direct_io, label_overrides);

    // Short sleep for delta-based sources to have meaningful deltas
    thread::sleep(Duration::from_millis(250));

    let mut map = HashMap::new();
    for source in &mut sources {
        for (id, reading) in source.poll() {
            map.insert(id, reading);
        }
    }
    map
}
