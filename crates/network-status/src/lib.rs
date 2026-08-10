//! Read-only Linux network interface inventory and byte counters.

use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkStatusError {
    #[error("failed to read network information: {0}")]
    Read(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkInterface {
    pub name: String,
    pub is_up: bool,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

pub fn collect() -> Result<Vec<NetworkInterface>, NetworkStatusError> {
    let stats = fs::read_to_string("/proc/net/dev")?;
    let mut result = Vec::new();
    for line in stats.lines().skip(2) {
        let Some((name, values)) = line.split_once(':') else { continue };
        let values: Vec<&str> = values.split_whitespace().collect();
        if values.len() < 9 { continue; }
        let rx_bytes = values[0].parse().unwrap_or(0);
        let tx_bytes = values[8].parse().unwrap_or(0);
        let name = name.trim().to_owned();
        let operstate = fs::read_to_string(format!("/sys/class/net/{name}/operstate"))
            .unwrap_or_default();
        result.push(NetworkInterface {
            name,
            is_up: operstate.trim() == "up",
            rx_bytes,
            tx_bytes,
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_network_interfaces() {
        let interfaces = collect().expect("network interfaces should be readable");
        assert!(!interfaces.is_empty());
        assert!(interfaces.iter().any(|interface| interface.name == "lo"));
    }
}
