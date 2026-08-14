//! Lightweight read-only Linux monitoring with bounded in-memory history.

use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fs, time::Instant};
use thiserror::Error;

const PROC_STAT: &str = "/proc/stat";
const MEM_INFO: &str = "/proc/meminfo";
const NET_DEV: &str = "/proc/net/dev";
const DISK_STATS: &str = "/proc/diskstats";
const HISTORY_LIMIT: usize = 120;

#[derive(Debug, Error)]
pub enum MonitoringError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: &'static str,
        source: std::io::Error,
    },
    #[error("invalid monitoring data in {path}: {value}")]
    Invalid { path: &'static str, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkRate {
    pub name: String,
    pub rx_bytes_per_second: f64,
    pub tx_bytes_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceBaseline {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub storage_read_bytes_per_second: f64,
    pub storage_write_bytes_per_second: f64,
    pub process_count: usize,
    pub running_processes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceDeviation {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub storage_read_bytes_per_second: f64,
    pub storage_write_bytes_per_second: f64,
    pub process_count: usize,
    pub running_processes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitorSnapshot {
    pub timestamp_ms: u128,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub swap_percent: f64,
    pub network: Vec<NetworkRate>,
    pub storage_read_bytes_per_second: f64,
    pub storage_write_bytes_per_second: f64,
    pub process_count: usize,
    pub running_processes: usize,
    pub baseline: Option<PerformanceBaseline>,
    pub deviation: Option<PerformanceDeviation>,
}

#[derive(Debug, Clone)]
struct CpuCounters {
    total: u64,
    idle: u64,
}

#[derive(Debug, Clone)]
struct NetworkCounters {
    rx: u64,
    tx: u64,
}

#[derive(Debug, Clone, Default)]
struct DiskCounters {
    read_sectors: u64,
    write_sectors: u64,
}

#[derive(Debug, Clone, Default)]
struct ProcessCounters {
    total: usize,
    running: usize,
}

#[derive(Debug)]
struct SampleState {
    cpu: CpuCounters,
    network: std::collections::HashMap<String, NetworkCounters>,
    disk: DiskCounters,
    process: ProcessCounters,
    at: Instant,
}

#[derive(Debug)]
pub struct Monitor {
    previous: Option<SampleState>,
    history: VecDeque<MonitorSnapshot>,
    started: Instant,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            previous: None,
            history: VecDeque::with_capacity(HISTORY_LIMIT),
            started: Instant::now(),
        }
    }
}

impl Monitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&mut self) -> Result<MonitorSnapshot, MonitoringError> {
        let cpu = read_cpu()?;
        let memory = read_memory()?;
        let network = read_network()?;
        let disk = read_disk()?;
        let process = read_processes()?;
        let now = Instant::now();
        let (cpu_percent, network_rates, storage_read, storage_write) = match &self.previous {
            Some(previous) => {
                let elapsed = now.duration_since(previous.at).as_secs_f64().max(0.001);
                (
                    cpu_usage(&previous.cpu, &cpu),
                    network_rates(&previous.network, &network, elapsed),
                    disk_rate(previous.disk.read_sectors, disk.read_sectors, elapsed),
                    disk_rate(previous.disk.write_sectors, disk.write_sectors, elapsed),
                )
            }
            None => (
                0.0,
                network
                    .keys()
                    .map(|name| NetworkRate {
                        name: name.clone(),
                        rx_bytes_per_second: 0.0,
                        tx_bytes_per_second: 0.0,
                    })
                    .collect(),
                0.0,
                0.0,
            ),
        };
        self.previous = Some(SampleState {
            cpu,
            network,
            disk,
            process: process.clone(),
            at: now,
        });

        let snapshot = MonitorSnapshot {
            timestamp_ms: self.started.elapsed().as_millis(),
            cpu_percent,
            memory_percent: percentage(memory.0, memory.1),
            swap_percent: percentage(memory.2, memory.3),
            network: network_rates,
            storage_read_bytes_per_second: storage_read,
            storage_write_bytes_per_second: storage_write,
            process_count: process.total,
            running_processes: process.running,
            baseline: self.performance_baseline(),
            deviation: None,
        };
        let deviation = snapshot
            .baseline
            .as_ref()
            .map(|baseline| deviation(&snapshot, baseline));
        let snapshot = MonitorSnapshot {
            deviation,
            ..snapshot
        };
        if self.history.len() == HISTORY_LIMIT {
            self.history.pop_front();
        }
        self.history.push_back(snapshot.clone());
        Ok(snapshot)
    }

    pub fn history(&self) -> Vec<MonitorSnapshot> {
        self.history.iter().cloned().collect()
    }

    pub fn performance_baseline(&self) -> Option<PerformanceBaseline> {
        if self.history.is_empty() {
            return None;
        }
        let count = self.history.len() as f64;
        Some(PerformanceBaseline {
            cpu_percent: self.history.iter().map(|s| s.cpu_percent).sum::<f64>() / count,
            memory_percent: self.history.iter().map(|s| s.memory_percent).sum::<f64>() / count,
            storage_read_bytes_per_second: self
                .history
                .iter()
                .map(|s| s.storage_read_bytes_per_second)
                .sum::<f64>()
                / count,
            storage_write_bytes_per_second: self
                .history
                .iter()
                .map(|s| s.storage_write_bytes_per_second)
                .sum::<f64>()
                / count,
            process_count: average_usize(self.history.iter().map(|s| s.process_count)),
            running_processes: average_usize(self.history.iter().map(|s| s.running_processes)),
        })
    }
}

fn read(path: &'static str) -> Result<String, MonitoringError> {
    fs::read_to_string(path).map_err(|source| MonitoringError::Read { path, source })
}

fn read_cpu() -> Result<CpuCounters, MonitoringError> {
    let line = read(PROC_STAT)?
        .lines()
        .next()
        .map(str::trim)
        .unwrap_or("")
        .to_owned();
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse().unwrap_or(0))
        .collect();
    if values.len() < 5 {
        return Err(MonitoringError::Invalid {
            path: PROC_STAT,
            value: line,
        });
    }
    Ok(CpuCounters {
        total: values.iter().sum(),
        idle: values[3].saturating_add(values.get(4).copied().unwrap_or(0)),
    })
}

fn read_memory() -> Result<(u64, u64, u64, u64), MonitoringError> {
    let content = read(MEM_INFO)?;
    let value = |key: &str| -> u64 {
        content
            .lines()
            .find(|line| line.starts_with(&format!("{key}:")))
            .and_then(|line| line.split_whitespace().nth(1)?.parse().ok())
            .unwrap_or(0)
    };
    Ok((
        value("MemTotal"),
        value("MemAvailable"),
        value("SwapTotal"),
        value("SwapFree"),
    ))
}

fn read_network() -> Result<std::collections::HashMap<String, NetworkCounters>, MonitoringError> {
    let content = read(NET_DEV)?;
    let mut result = std::collections::HashMap::new();
    for line in content.lines().skip(2) {
        let Some((name, values)) = line.split_once(':') else {
            continue;
        };
        let values: Vec<&str> = values.split_whitespace().collect();
        if values.len() < 9 {
            continue;
        }
        result.insert(
            name.trim().to_owned(),
            NetworkCounters {
                rx: values[0].parse().unwrap_or(0),
                tx: values[8].parse().unwrap_or(0),
            },
        );
    }
    Ok(result)
}

fn read_disk() -> Result<DiskCounters, MonitoringError> {
    let content = read(DISK_STATS)?;
    let mut result = DiskCounters::default();
    for line in content.lines() {
        let values: Vec<&str> = line.split_whitespace().collect();
        if values.len() < 14 {
            continue;
        }
        result.read_sectors = result
            .read_sectors
            .saturating_add(values[5].parse::<u64>().unwrap_or(0));
        result.write_sectors = result
            .write_sectors
            .saturating_add(values[9].parse::<u64>().unwrap_or(0));
    }
    Ok(result)
}

fn read_processes() -> Result<ProcessCounters, MonitoringError> {
    let mut result = ProcessCounters::default();
    for entry in fs::read_dir("/proc")? {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let status = match fs::read_to_string(entry.path().join("status")) {
            Ok(value) => value,
            Err(_) => continue,
        };
        result.total += 1;
        if status
            .lines()
            .find_map(|line| line.strip_prefix("State:"))
            .is_some_and(|state| state.trim_start().starts_with('R'))
        {
            result.running += 1;
        }
    }
    Ok(result)
}

fn cpu_usage(previous: &CpuCounters, current: &CpuCounters) -> f64 {
    let total = current.total.saturating_sub(previous.total);
    let idle = current.idle.saturating_sub(previous.idle);
    if total == 0 {
        0.0
    } else {
        (100.0 * (total.saturating_sub(idle) as f64) / total as f64).clamp(0.0, 100.0)
    }
}

fn disk_rate(previous: u64, current: u64, seconds: f64) -> f64 {
    current.saturating_sub(previous).saturating_mul(512) as f64 / seconds
}

fn network_rates(
    previous: &std::collections::HashMap<String, NetworkCounters>,
    current: &std::collections::HashMap<String, NetworkCounters>,
    seconds: f64,
) -> Vec<NetworkRate> {
    let mut rates: Vec<_> = current
        .iter()
        .map(|(name, now)| {
            let old = previous.get(name).cloned().unwrap_or(NetworkCounters {
                rx: now.rx,
                tx: now.tx,
            });
            NetworkRate {
                name: name.clone(),
                rx_bytes_per_second: now.rx.saturating_sub(old.rx) as f64 / seconds,
                tx_bytes_per_second: now.tx.saturating_sub(old.tx) as f64 / seconds,
            }
        })
        .collect();
    rates.sort_by(|a, b| a.name.cmp(&b.name));
    rates
}

fn percentage(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (100.0 * (total.saturating_sub(used) as f64) / total as f64).clamp(0.0, 100.0)
    }
}

fn average_usize(values: impl Iterator<Item = usize>) -> usize {
    let values: Vec<_> = values.collect();
    if values.is_empty() {
        0
    } else {
        ((values.iter().sum::<usize>() as f64 / values.len() as f64).round()) as usize
    }
}

fn deviation(snapshot: &MonitorSnapshot, baseline: &PerformanceBaseline) -> PerformanceDeviation {
    PerformanceDeviation {
        cpu_percent: snapshot.cpu_percent - baseline.cpu_percent,
        memory_percent: snapshot.memory_percent - baseline.memory_percent,
        storage_read_bytes_per_second: snapshot.storage_read_bytes_per_second
            - baseline.storage_read_bytes_per_second,
        storage_write_bytes_per_second: snapshot.storage_write_bytes_per_second
            - baseline.storage_write_bytes_per_second,
        process_count: snapshot.process_count.abs_diff(baseline.process_count),
        running_processes: snapshot
            .running_processes
            .abs_diff(baseline.running_processes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_usage_is_bounded() {
        let previous = CpuCounters {
            total: 100,
            idle: 40,
        };
        let current = CpuCounters {
            total: 200,
            idle: 80,
        };
        assert_eq!(cpu_usage(&previous, &current), 60.0);
    }

    #[test]
    fn monitor_keeps_bounded_history() {
        let mut monitor = Monitor::new();
        for _ in 0..(HISTORY_LIMIT + 5) {
            let _ = monitor.snapshot();
        }
        assert!(monitor.history().len() <= HISTORY_LIMIT);
    }

    #[test]
    fn disk_rate_converts_sectors_to_bytes_per_second() {
        assert_eq!(disk_rate(100, 200, 2.0), 25_600.0);
    }

    #[test]
    fn baseline_is_bounded_by_history() {
        let mut monitor = Monitor::new();
        for _ in 0..3 {
            let _ = monitor.snapshot();
        }
        assert!(monitor.performance_baseline().is_some());
    }
}
