//! Bounded, read-only Linux service intelligence.
//!
//! Service inspection uses a fixed `systemctl` invocation with no shell and
//! never starts, stops, restarts, enables, disables, or otherwise mutates a
//! service.

use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

const DEFAULT_MAX_SERVICES: usize = 100;
const DEFAULT_TOP_N: usize = 20;

#[derive(Debug, Error)]
pub enum ServiceIntelligenceError {
    #[error("failed to invoke systemctl: {0}")]
    Command(#[source] std::io::Error),
    #[error("systemctl returned an error: {0}")]
    Systemctl(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceObservation {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub unit_file_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceAnomaly {
    pub kind: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceAnalysis {
    pub services_scanned: usize,
    pub truncated: bool,
    pub failed_services: usize,
    pub inactive_services: usize,
    pub services: Vec<ServiceObservation>,
    pub anomalies: Vec<ServiceAnomaly>,
}

#[derive(Debug, Clone, Copy)]
pub struct AnalysisLimits {
    pub max_services: usize,
    pub top_n: usize,
}

impl Default for AnalysisLimits {
    fn default() -> Self {
        Self {
            max_services: DEFAULT_MAX_SERVICES,
            top_n: DEFAULT_TOP_N,
        }
    }
}

/// Analyze systemd services using bounded, read-only inspection.
pub fn analyze() -> Result<ServiceAnalysis, ServiceIntelligenceError> {
    analyze_with_limits(AnalysisLimits::default())
}

/// Analyze systemd services with explicit bounds.
pub fn analyze_with_limits(
    limits: AnalysisLimits,
) -> Result<ServiceAnalysis, ServiceIntelligenceError> {
    let output = Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--all",
            "--no-legend",
            "--no-pager",
        ])
        .output()
        .map_err(ServiceIntelligenceError::Command)?;

    if !output.status.success() {
        return Err(ServiceIntelligenceError::Systemctl(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    let mut services = Vec::new();
    let mut truncated = false;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if services.len() >= limits.max_services {
            truncated = true;
            break;
        }
        if let Some(service) = parse_unit_line(line) {
            services.push(service);
        }
    }

    let failed_services = services
        .iter()
        .filter(|service| service.active_state == "failed")
        .count();
    let inactive_services = services
        .iter()
        .filter(|service| service.active_state == "inactive")
        .count();

    let mut anomalies = Vec::new();
    if failed_services > 0 {
        anomalies.push(ServiceAnomaly {
            kind: "failed-services".into(),
            description: format!("Observed {failed_services} failed service(s)."),
        });
    }
    if inactive_services > limits.top_n {
        anomalies.push(ServiceAnomaly {
            kind: "inactive-services".into(),
            description: format!(
                "Observed {inactive_services} inactive service(s) in the bounded inventory."
            ),
        });
    }
    anomalies.truncate(limits.top_n);

    Ok(ServiceAnalysis {
        services_scanned: services.len(),
        truncated,
        failed_services,
        inactive_services,
        services,
        anomalies,
    })
}

fn parse_unit_line(line: &str) -> Option<ServiceObservation> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 4 {
        return None;
    }
    let name = fields[0];
    if !name.ends_with(".service") {
        return None;
    }
    let description = fields[4..].join(" ");
    Some(ServiceObservation {
        name: name.to_owned(),
        description,
        load_state: fields[1].to_owned(),
        active_state: fields[2].to_owned(),
        sub_state: fields[3].to_owned(),
        unit_file_state: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_service_inventory_line() {
        let service =
            parse_unit_line("ssh.service loaded active running OpenSSH server daemon").unwrap();
        assert_eq!(service.name, "ssh.service");
        assert_eq!(service.active_state, "active");
        assert_eq!(service.sub_state, "running");
        assert_eq!(service.description, "OpenSSH server daemon");
    }

    #[test]
    fn ignores_non_service_units() {
        assert!(
            parse_unit_line("multi-user.target loaded active active Multi-User System").is_none()
        );
    }
}
