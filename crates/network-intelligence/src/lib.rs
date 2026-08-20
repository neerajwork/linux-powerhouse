//! Bounded, read-only Linux network analysis.
//!
//! This crate enriches interface status with socket-level observations from
//! procfs. It never changes routes, firewalls, sockets, or network settings.

use serde::{Deserialize, Serialize};
use std::{fs, io, net::Ipv4Addr, path::Path};
use thiserror::Error;

const TCP: &str = "/proc/net/tcp";
const TCP6: &str = "/proc/net/tcp6";
const DEFAULT_MAX_SOCKETS: usize = 1000;
const DEFAULT_TOP_N: usize = 20;

#[derive(Debug, Error)]
pub enum NetworkIntelligenceError {
    #[error("failed to read network information from {path}: {source}")]
    Read { path: String, source: io::Error },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SocketProtocol {
    Tcp,
    Tcp6,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SocketObservation {
    pub protocol: SocketProtocol,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub uid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkAnomaly {
    pub kind: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkAnalysis {
    pub sockets_scanned: usize,
    pub skipped_entries: usize,
    pub truncated: bool,
    pub listening_ports: Vec<SocketObservation>,
    pub established_connections: usize,
    pub anomalies: Vec<NetworkAnomaly>,
}

#[derive(Debug, Clone, Copy)]
pub struct AnalysisLimits {
    pub max_sockets: usize,
    pub top_n: usize,
}

impl Default for AnalysisLimits {
    fn default() -> Self {
        Self {
            max_sockets: DEFAULT_MAX_SOCKETS,
            top_n: DEFAULT_TOP_N,
        }
    }
}

/// Analyze TCP sockets using bounded, read-only procfs inspection.
pub fn analyze() -> Result<NetworkAnalysis, NetworkIntelligenceError> {
    analyze_with_limits(AnalysisLimits::default())
}

/// Analyze TCP sockets with explicit bounds.
pub fn analyze_with_limits(
    limits: AnalysisLimits,
) -> Result<NetworkAnalysis, NetworkIntelligenceError> {
    let mut observations = Vec::new();
    let mut skipped_entries = 0usize;
    let mut truncated = false;

    for (path, protocol) in [(TCP, SocketProtocol::Tcp), (TCP6, SocketProtocol::Tcp6)] {
        let content =
            fs::read_to_string(path).map_err(|source| NetworkIntelligenceError::Read {
                path: path.into(),
                source,
            })?;
        for line in content.lines().skip(1) {
            if observations.len() >= limits.max_sockets {
                truncated = true;
                break;
            }
            match parse_socket_line(line, protocol) {
                Some(socket) => observations.push(socket),
                None => skipped_entries += 1,
            }
        }
        if truncated {
            break;
        }
    }

    let established_connections = observations.iter().filter(|s| s.state == "01").count();
    let mut listening_ports: Vec<_> = observations
        .iter()
        .filter(|socket| socket.state == "0A")
        .cloned()
        .collect();
    listening_ports.sort_by(|a, b| {
        a.local_port
            .cmp(&b.local_port)
            .then_with(|| a.local_address.cmp(&b.local_address))
    });
    listening_ports.truncate(limits.top_n);

    let mut anomalies = Vec::new();
    if listening_ports.iter().any(|socket| socket.local_port == 0) {
        anomalies.push(NetworkAnomaly {
            kind: "invalid-listener".into(),
            description: "A listening socket reported port 0.".into(),
        });
    }
    if established_connections > limits.top_n * 10 {
        anomalies.push(NetworkAnomaly {
            kind: "connection-volume".into(),
            description: format!(
                "Observed {established_connections} established TCP connections in the bounded scan."
            ),
        });
    }
    anomalies.truncate(limits.top_n);

    Ok(NetworkAnalysis {
        sockets_scanned: observations.len(),
        skipped_entries,
        truncated,
        listening_ports,
        established_connections,
        anomalies,
    })
}

fn parse_socket_line(line: &str, protocol: SocketProtocol) -> Option<SocketObservation> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 8 {
        return None;
    }
    let (local_address, local_port) = parse_endpoint(fields[1], protocol)?;
    let (remote_address, remote_port) = parse_endpoint(fields[2], protocol)?;
    let uid = fields[7].parse::<u32>().ok()?;
    Some(SocketObservation {
        protocol,
        local_address,
        local_port,
        remote_address,
        remote_port,
        state: fields[3].to_owned(),
        uid,
    })
}

fn parse_endpoint(value: &str, protocol: SocketProtocol) -> Option<(String, u16)> {
    let (address, port) = value.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;
    match protocol {
        SocketProtocol::Tcp => {
            if address.len() != 8 {
                return None;
            }
            let bytes = u32::from_str_radix(address, 16).ok()?.to_le_bytes();
            Some((Ipv4Addr::from(bytes).to_string(), port))
        }
        SocketProtocol::Tcp6 => Some((format_ipv6(address)?, port)),
    }
}

fn format_ipv6(address: &str) -> Option<String> {
    if address.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (index, chunk) in address.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let text = std::str::from_utf8(chunk).ok()?;
        bytes[index] = u8::from_str_radix(text, 16).ok()?;
    }
    let address = std::net::Ipv6Addr::from(bytes);
    Some(address.to_string())
}

#[allow(dead_code)]
fn path_exists(path: &Path) -> bool {
    path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_analysis_reads_linux_tcp_tables() {
        let result = analyze_with_limits(AnalysisLimits {
            max_sockets: 100,
            top_n: 10,
        })
        .expect("TCP tables should be readable on Linux CI");
        assert!(result.sockets_scanned <= 100);
        assert!(result.listening_ports.len() <= 10);
        assert!(result.anomalies.len() <= 10);
    }

    #[test]
    fn parses_ipv4_tcp_endpoint() {
        let endpoint = parse_endpoint("0100007F:1F90", SocketProtocol::Tcp).unwrap();
        assert_eq!(endpoint, ("127.0.0.1".to_owned(), 8080));
    }
}
