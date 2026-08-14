//! Bounded, read-only process analysis for Linux.
//!
//! This crate enriches the basic process inventory with parent/child
//! relationships, cumulative CPU ticks, memory usage, and deterministic
//! anomaly signals. It never terminates, pauses, or mutates processes.

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::Path};
use thiserror::Error;

const PROC: &str = "/proc";
const MEM_INFO: &str = "/proc/meminfo";
const DEFAULT_MAX_PROCESSES: usize = 500;
const DEFAULT_TOP_N: usize = 20;
const HIGH_MEMORY_PERCENT: f64 = 5.0;

#[derive(Debug, Error)]
pub enum ProcessIntelligenceError {
    #[error("failed to read {path}: {source}")]
    Read { path: String, source: io::Error },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessInsight {
    pub pid: u32,
    pub name: String,
    pub state: String,
    pub parent_pid: u32,
    pub memory_bytes: u64,
    pub memory_percent: f64,
    pub cpu_time_ticks: u64,
    pub child_count: usize,
    pub anomaly: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessAnalysis {
    pub total_processes: usize,
    pub entries_scanned: usize,
    pub skipped_entries: usize,
    pub truncated: bool,
    pub zombie_count: usize,
    pub top_consumers: Vec<ProcessInsight>,
    pub top_cpu_consumers: Vec<ProcessInsight>,
    pub anomalies: Vec<ProcessInsight>,
}

#[derive(Debug, Clone, Copy)]
pub struct AnalysisLimits {
    pub max_processes: usize,
    pub top_n: usize,
}

impl Default for AnalysisLimits {
    fn default() -> Self {
        Self {
            max_processes: DEFAULT_MAX_PROCESSES,
            top_n: DEFAULT_TOP_N,
        }
    }
}

#[derive(Debug, Clone)]
struct RawProcess {
    pid: u32,
    name: String,
    state: String,
    parent_pid: u32,
    memory_bytes: u64,
    cpu_time_ticks: u64,
}

/// Analyze a bounded snapshot of Linux processes.
pub fn analyze() -> Result<ProcessAnalysis, ProcessIntelligenceError> {
    analyze_with_limits(AnalysisLimits::default())
}

/// Analyze processes with explicit bounds.
pub fn analyze_with_limits(
    limits: AnalysisLimits,
) -> Result<ProcessAnalysis, ProcessIntelligenceError> {
    let memory_total = read_memory_total()?;
    let mut processes = Vec::new();
    let mut skipped_entries = 0usize;
    let mut entries_scanned = 0usize;
    let mut truncated = false;

    for entry in fs::read_dir(PROC).map_err(|source| ProcessIntelligenceError::Read {
        path: PROC.into(),
        source,
    })? {
        if processes.len() >= limits.max_processes {
            truncated = true;
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        entries_scanned += 1;
        let pid = match name.parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };
        match read_process(&entry.path(), pid) {
            Ok(process) => processes.push(process),
            Err(_) => skipped_entries += 1,
        }
    }

    let mut child_counts = HashMap::<u32, usize>::new();
    for process in &processes {
        *child_counts.entry(process.parent_pid).or_default() += 1;
    }

    let mut insights: Vec<ProcessInsight> = processes
        .into_iter()
        .map(|process| {
            let memory_percent = if memory_total == 0 {
                0.0
            } else {
                100.0 * process.memory_bytes as f64 / memory_total as f64
            };
            let child_count = child_counts.get(&process.pid).copied().unwrap_or(0);
            let anomaly = if process.state.starts_with('Z') {
                Some("zombie process".to_owned())
            } else if memory_percent >= HIGH_MEMORY_PERCENT {
                Some(format!("high memory usage ({memory_percent:.1}%)"))
            } else {
                None
            };
            ProcessInsight {
                pid: process.pid,
                name: process.name,
                state: process.state,
                parent_pid: process.parent_pid,
                memory_bytes: process.memory_bytes,
                memory_percent,
                cpu_time_ticks: process.cpu_time_ticks,
                child_count,
                anomaly,
            }
        })
        .collect();

    let zombie_count = insights
        .iter()
        .filter(|process| process.state.starts_with('Z'))
        .count();
    let total_processes = insights.len();

    insights.sort_by(|a, b| {
        b.memory_bytes
            .cmp(&a.memory_bytes)
            .then_with(|| a.pid.cmp(&b.pid))
    });
    let top_consumers = insights.iter().take(limits.top_n).cloned().collect();

    let mut top_cpu = insights.clone();
    top_cpu.sort_by(|a, b| {
        b.cpu_time_ticks
            .cmp(&a.cpu_time_ticks)
            .then_with(|| a.pid.cmp(&b.pid))
    });
    let top_cpu_consumers = top_cpu.into_iter().take(limits.top_n).collect();

    let mut anomalies: Vec<_> = insights
        .iter()
        .filter(|process| process.anomaly.is_some())
        .cloned()
        .collect();
    anomalies.truncate(limits.top_n);

    Ok(ProcessAnalysis {
        total_processes,
        entries_scanned,
        skipped_entries,
        truncated,
        zombie_count,
        top_consumers,
        top_cpu_consumers,
        anomalies,
    })
}

fn read_process(path: &Path, pid: u32) -> Result<RawProcess, io::Error> {
    let status = fs::read_to_string(path.join("status"))?;
    let stat = fs::read_to_string(path.join("stat"))?;
    let name = field(&status, "Name:").unwrap_or_else(|| "unknown".into());
    let memory_bytes = field(&status, "VmRSS:")
        .and_then(|value| value.split_whitespace().next()?.parse::<u64>().ok())
        .unwrap_or(0)
        .saturating_mul(1024);

    let (_, fields) = stat
        .split_once(')')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid /proc/<pid>/stat"))?;
    let fields: Vec<&str> = fields.split_whitespace().collect();
    if fields.len() < 20 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incomplete /proc/<pid>/stat",
        ));
    }
    let state = fields[0].to_owned();
    let parent_pid = fields[1].parse::<u32>().unwrap_or(0);
    let utime = fields[11].parse::<u64>().unwrap_or(0);
    let stime = fields[12].parse::<u64>().unwrap_or(0);

    Ok(RawProcess {
        pid,
        name,
        state,
        parent_pid,
        memory_bytes,
        cpu_time_ticks: utime.saturating_add(stime),
    })
}

fn read_memory_total() -> Result<u64, ProcessIntelligenceError> {
    let content =
        fs::read_to_string(MEM_INFO).map_err(|source| ProcessIntelligenceError::Read {
            path: MEM_INFO.into(),
            source,
        })?;
    let kilobytes = content
        .lines()
        .find(|line| line.starts_with("MemTotal:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    Ok(kilobytes.saturating_mul(1024))
}

fn field(content: &str, key: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(key).map(|value| value.trim().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_analysis_returns_processes() {
        let result = analyze_with_limits(AnalysisLimits {
            max_processes: 20,
            top_n: 5,
        })
        .expect("process analysis should be readable on Linux CI");
        assert!(result.total_processes > 0);
        assert!(result.entries_scanned <= 20);
        assert!(result.top_consumers.len() <= 5);
        assert!(result.top_cpu_consumers.len() <= 5);
        assert!(result.anomalies.len() <= 5);
    }

    #[test]
    fn top_cpu_consumers_are_sorted() {
        let result = analyze_with_limits(AnalysisLimits {
            max_processes: 20,
            top_n: 5,
        })
        .expect("process analysis should be readable on Linux CI");
        assert!(
            result
                .top_cpu_consumers
                .windows(2)
                .all(|pair| pair[0].cpu_time_ticks >= pair[1].cpu_time_ticks)
        );
    }

    #[test]
    fn invalid_stat_is_rejected() {
        let path =
            std::path::PathBuf::from("/tmp/linux-powerhouse-process-intelligence-no-such-pid");
        let result = read_process(&path, 1);
        assert!(result.is_err());
    }
}
