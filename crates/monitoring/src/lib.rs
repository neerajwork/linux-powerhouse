//! Lightweight read-only Linux monitoring with bounded in-memory history.

use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs,
    time::Instant,
};
use thiserror::Error;

const PROC_STAT: &str = "/proc/stat";
const MEM_INFO: &str = "/proc/meminfo";
const NET_DEV: &str = "/proc/net/dev";
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
pub struct MonitorSnapshot {
    pub timestamp_ms: u128,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub swap_percent: f64,
    pub network: Vec<NetworkRate>,
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

#[derive(Debug)]
struct SampleState {
    cpu: CpuCounters,
    network: std::collections::HashMap<String, NetworkCounters>,
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
        let now = Instant::now();
        let (cpu_percent, network_rates) = match &self.previous {
            Some(previous) => {
                let elapsed = now.duration_since(previous.at).as_secs_f64().max(0.001);
                (
                    cpu_usage(&previous.cpu, &cpu),
                    network_rates(&previous.network, &network, elapsed),
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
            ),
        };
        self.previous = Some(SampleState {
            cpu,
            network,
            at: now,
        });
        let snapshot = MonitorSnapshot {
            timestamp_ms: self.started.elapsed().as_millis(),
            cpu_percent,
            memory_percent: percentage(memory.0, memory.1),
            swap_percent: percentage(memory.2, memory.3),
            network: network_rates,
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

fn cpu_usage(previous: &CpuCounters, current: &CpuCounters) -> f64 {
    let total = current.total.saturating_sub(previous.total);
    let idle = current.idle.saturating_sub(previous.idle);
    if total == 0 {
        0.0
    } else {
        (100.0 * (total.saturating_sub(idle) as f64) / total as f64).clamp(0.0, 100.0)
    }
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
}
