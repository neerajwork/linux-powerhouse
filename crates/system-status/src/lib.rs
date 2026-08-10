//! Read-only Linux system status collection.
//!
//! This backend intentionally uses kernel-provided interfaces instead of
//! invoking shell commands. It is therefore suitable for the Powerhouse
//! execution boundary and can later be exposed through the Tool Registry.

use serde::{Deserialize, Serialize};
use thiserror::Error;

const OS_RELEASE_PATH: &str = "/etc/os-release";
const KERNEL_RELEASE_PATH: &str = "/proc/sys/kernel/osrelease";
const CPU_INFO_PATH: &str = "/proc/cpuinfo";
const MEM_INFO_PATH: &str = "/proc/meminfo";
const UPTIME_PATH: &str = "/proc/uptime";
const HOSTNAME_PATH: &str = "/etc/hostname";

#[derive(Debug, Error)]
pub enum SystemStatusError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: &'static str,
        source: std::io::Error,
    },
    #[error("invalid value in {path}: {value}")]
    InvalidValue { path: &'static str, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemStatus {
    pub operating_system: OperatingSystem,
    pub kernel_version: String,
    pub architecture: String,
    pub hostname: String,
    pub cpu_model: Option<String>,
    pub cpu_logical_cores: usize,
    pub memory_total_bytes: u64,
    pub memory_available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_free_bytes: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatingSystem {
    pub id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

/// Collect a snapshot of read-only system information from Linux kernel and
/// operating-system interfaces. No shell commands or elevated privileges are
/// required.
pub fn collect() -> Result<SystemStatus, SystemStatusError> {
    let os_release = read(OS_RELEASE_PATH)?;
    let mem_info = read(MEM_INFO_PATH)?;
    let cpu_info = read(CPU_INFO_PATH)?;
    let kernel_version = read(KERNEL_RELEASE_PATH)?.trim().to_owned();
    let hostname = read(HOSTNAME_PATH)?.trim().to_owned();
    let uptime_seconds = parse_uptime_seconds(&read(UPTIME_PATH)?)?;

    let operating_system = OperatingSystem {
        id: os_release_value(&os_release, "ID").map(str::to_owned),
        name: os_release_value(&os_release, "PRETTY_NAME")
            .or_else(|| os_release_value(&os_release, "NAME"))
            .map(str::to_owned),
        version: os_release_value(&os_release, "VERSION_ID").map(str::to_owned),
    };

    let cpu_model = cpu_info.lines().find_map(|line| {
        line.strip_prefix("model name\t:")
            .map(str::trim)
            .map(str::to_owned)
    });
    let cpu_logical_cores = cpu_info
        .lines()
        .filter(|line| line.starts_with("processor\t:"))
        .count()
        .max(1);

    Ok(SystemStatus {
        operating_system,
        kernel_version,
        architecture: std::env::consts::ARCH.to_owned(),
        hostname,
        cpu_model,
        cpu_logical_cores,
        memory_total_bytes: meminfo_kib(&mem_info, "MemTotal")? * 1024,
        memory_available_bytes: meminfo_kib(&mem_info, "MemAvailable")? * 1024,
        swap_total_bytes: meminfo_kib(&mem_info, "SwapTotal")? * 1024,
        swap_free_bytes: meminfo_kib(&mem_info, "SwapFree")? * 1024,
        uptime_seconds,
    })
}

fn read(path: &'static str) -> Result<String, SystemStatusError> {
    std::fs::read_to_string(path).map_err(|source| SystemStatusError::Read { path, source })
}

fn os_release_value<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    content.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        if candidate != key {
            return None;
        }
        Some(value.trim_matches('"'))
    })
}

fn meminfo_kib(content: &str, key: &str) -> Result<u64, SystemStatusError> {
    let line = content
        .lines()
        .find(|line| line.starts_with(&format!("{key}:")))
        .ok_or_else(|| SystemStatusError::InvalidValue {
            path: MEM_INFO_PATH,
            value: key.to_owned(),
        })?;

    let value = line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| SystemStatusError::InvalidValue {
            path: MEM_INFO_PATH,
            value: line.to_owned(),
        })?;

    value
        .parse::<u64>()
        .map_err(|_| SystemStatusError::InvalidValue {
            path: MEM_INFO_PATH,
            value: line.to_owned(),
        })
}

fn parse_uptime_seconds(content: &str) -> Result<u64, SystemStatusError> {
    let seconds = content
        .split_whitespace()
        .next()
        .ok_or_else(|| SystemStatusError::InvalidValue {
            path: UPTIME_PATH,
            value: content.to_owned(),
        })?
        .parse::<f64>()
        .map_err(|_| SystemStatusError::InvalidValue {
            path: UPTIME_PATH,
            value: content.to_owned(),
        })?;

    if !seconds.is_finite() || seconds < 0.0 {
        return Err(SystemStatusError::InvalidValue {
            path: UPTIME_PATH,
            value: content.to_owned(),
        });
    }

    Ok(seconds as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_os_release_values() {
        let content = "NAME=Linux\nID=example\nPRETTY_NAME=\"Example Linux\"\nVERSION_ID=\"1.2\"\n";
        assert_eq!(os_release_value(content, "ID"), Some("example"));
        assert_eq!(
            os_release_value(content, "PRETTY_NAME"),
            Some("Example Linux")
        );
        assert_eq!(os_release_value(content, "VERSION_ID"), Some("1.2"));
    }

    #[test]
    fn parses_memory_values_in_kib() {
        let content = "MemTotal:       16384 kB\nMemAvailable:    8192 kB\n";
        assert_eq!(meminfo_kib(content, "MemTotal").unwrap(), 16384);
        assert_eq!(meminfo_kib(content, "MemAvailable").unwrap(), 8192);
    }

    #[test]
    fn parses_uptime_as_whole_seconds() {
        assert_eq!(parse_uptime_seconds("123.75 456.0\n").unwrap(), 123);
    }
}
