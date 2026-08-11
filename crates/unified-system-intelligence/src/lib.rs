//! Unified, read-only system intelligence.
//!
//! This crate provides a stable aggregate snapshot over the existing
//! intelligence modules. It does not mutate the host system.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnifiedSystemIntelligenceError {
    #[error("storage intelligence failed: {0}")]
    Storage(String),
    #[error("process intelligence failed: {0}")]
    Process(String),
    #[error("network intelligence failed: {0}")]
    Network(String),
    #[error("service intelligence failed: {0}")]
    Service(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthLevel {
    Healthy,
    Attention,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemIntelligenceSnapshot {
    pub health: HealthLevel,
    pub storage_anomalies: usize,
    pub process_anomalies: usize,
    pub network_anomalies: usize,
    pub service_anomalies: usize,
    pub total_anomalies: usize,
}

/// Build one bounded, deterministic snapshot from the existing intelligence layers.
///
/// The storage root is supplied explicitly because storage analysis is intentionally
/// scoped to a caller-approved path.
pub fn snapshot(
    storage_root: impl AsRef<Path>,
) -> Result<SystemIntelligenceSnapshot, UnifiedSystemIntelligenceError> {
    let storage = storage_intelligence::analyze(storage_root)
        .map_err(|error| UnifiedSystemIntelligenceError::Storage(error.to_string()))?;
    let process = process_intelligence::analyze()
        .map_err(|error| UnifiedSystemIntelligenceError::Process(error.to_string()))?;
    let network = network_intelligence::analyze()
        .map_err(|error| UnifiedSystemIntelligenceError::Network(error.to_string()))?;
    let service = service_intelligence::analyze()
        .map_err(|error| UnifiedSystemIntelligenceError::Service(error.to_string()))?;

    let storage_anomalies = usize::from(storage.truncated) + usize::from(storage.skipped_entries > 0);
    let process_anomalies = process.anomalies.len();
    let network_anomalies = network.anomalies.len();
    let service_anomalies = service.anomalies.len();
    let total_anomalies =
        storage_anomalies + process_anomalies + network_anomalies + service_anomalies;

    let health = match total_anomalies {
        0 => HealthLevel::Healthy,
        1..=3 => HealthLevel::Attention,
        _ => HealthLevel::Degraded,
    };

    Ok(SystemIntelligenceSnapshot {
        health,
        storage_anomalies,
        process_anomalies,
        network_anomalies,
        service_anomalies,
        total_anomalies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_thresholds_are_deterministic() {
        assert_eq!(health_for(0), HealthLevel::Healthy);
        assert_eq!(health_for(3), HealthLevel::Attention);
        assert_eq!(health_for(4), HealthLevel::Degraded);
    }

    fn health_for(total: usize) -> HealthLevel {
        match total {
            0 => HealthLevel::Healthy,
            1..=3 => HealthLevel::Attention,
            _ => HealthLevel::Degraded,
        }
    }
}
