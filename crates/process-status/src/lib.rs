//! Read-only Linux process inventory from `/proc`.

use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessStatusError {
    #[error("failed to read process information: {0}")]
    Read(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub state: String,
    pub memory_bytes: u64,
}

/// Collect a bounded process snapshot. Processes are sorted by memory usage.
pub fn collect(limit: usize) -> Result<Vec<ProcessInfo>, ProcessStatusError> {
    let mut processes = Vec::new();
    for entry in fs::read_dir("/proc")? {
        let entry = match entry { Ok(value) => value, Err(_) => continue };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_digit()) { continue; }
        let pid = match name.parse::<u32>() { Ok(value) => value, Err(_) => continue };
        let status_path = entry.path().join("status");
        let content = match fs::read_to_string(status_path) { Ok(value) => value, Err(_) => continue };
        let process_name = field(&content, "Name:").unwrap_or_else(|| "unknown".into());
        let state = field(&content, "State:").unwrap_or_else(|| "unknown".into());
        let memory_bytes = field(&content, "VmRSS:")
            .and_then(|value| value.split_whitespace().next()?.parse::<u64>().ok())
            .unwrap_or(0)
            .saturating_mul(1024);
        processes.push(ProcessInfo { pid, name: process_name, state, memory_bytes });
    }
    processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    processes.truncate(limit);
    Ok(processes)
}

fn field(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| line.strip_prefix(key).map(|value| value.trim().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_process_snapshot() {
        let processes = collect(10).expect("process inventory should be readable");
        assert!(!processes.is_empty());
        assert!(processes.len() <= 10);
    }
}
