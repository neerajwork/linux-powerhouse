//! Deterministic, read-only system health signals derived from trusted metrics.

use monitoring::MonitorSnapshot;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod alert_action_preview;
pub mod alert_correlation;
pub mod alert_explanation;
pub mod alert_guidance;
pub mod alert_history;
pub mod alerts;
pub mod performance;
pub use alert_action_preview::{AlertActionPreview, preview_alert_actions};
pub use alert_correlation::{AlertPerformanceCorrelation, correlate_alert};
pub use alert_explanation::{AlertExplanation, explain_alert};
pub use alert_guidance::{AlertGuidance, guide_alert};
pub use alert_history::{
    AlertEvent, AlertEventHistory, AlertEventReason, AlertProcessEvidence,
    DEFAULT_ALERT_HISTORY_LIMIT, create_event as create_alert_event,
    event_reason as alert_event_reason,
};
pub use alerts::{AlertDecision, AlertPolicy, AlertSeverity, AlertState, alert_decision};
pub use performance::{
    PerformanceAnomaly, PerformanceAnomalyLevel, PerformanceAnomalyReport, PerformanceMetric,
    explain as explain_performance,
};

const CPU_WARNING: f64 = 80.0;
const CPU_CRITICAL: f64 = 95.0;
const MEMORY_WARNING: f64 = 80.0;
const MEMORY_CRITICAL: f64 = 90.0;
const SWAP_WARNING: f64 = 20.0;
const SWAP_CRITICAL: f64 = 50.0;
const STORAGE_WARNING: u8 = 80;
const STORAGE_CRITICAL: u8 = 90;

#[derive(Debug, Error)]
pub enum HealthStatusError {
    #[error("no monitoring snapshot is available")]
    NoSnapshot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthLevel {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalKind {
    Cpu,
    Memory,
    Swap,
    Storage,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthSignal {
    pub kind: SignalKind,
    pub level: HealthLevel,
    pub value: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthSnapshot {
    pub overall: HealthLevel,
    pub signals: Vec<HealthSignal>,
}

/// Evaluate deterministic health signals from the latest monitoring snapshot.
/// Storage can be supplied separately because it is not part of the monitoring stream.
pub fn evaluate(
    monitoring: Option<&MonitorSnapshot>,
    max_storage_usage: Option<u8>,
) -> Result<HealthSnapshot, HealthStatusError> {
    let snapshot = monitoring.ok_or(HealthStatusError::NoSnapshot)?;
    let mut signals = vec![
        threshold_signal(
            SignalKind::Cpu,
            snapshot.cpu_percent,
            CPU_WARNING,
            CPU_CRITICAL,
            "CPU utilization",
        ),
        threshold_signal(
            SignalKind::Memory,
            snapshot.memory_percent,
            MEMORY_WARNING,
            MEMORY_CRITICAL,
            "Memory utilization",
        ),
        threshold_signal(
            SignalKind::Swap,
            snapshot.swap_percent,
            SWAP_WARNING,
            SWAP_CRITICAL,
            "Swap utilization",
        ),
    ];

    if let Some(usage) = max_storage_usage {
        signals.push(threshold_signal(
            SignalKind::Storage,
            usage as f64,
            STORAGE_WARNING as f64,
            STORAGE_CRITICAL as f64,
            "Filesystem utilization",
        ));
    }

    let overall = signals
        .iter()
        .map(|signal| signal.level)
        .max_by_key(level_rank)
        .unwrap_or(HealthLevel::Healthy);

    Ok(HealthSnapshot { overall, signals })
}

fn threshold_signal(
    kind: SignalKind,
    value: f64,
    warning: f64,
    critical: f64,
    label: &str,
) -> HealthSignal {
    let level = if value >= critical {
        HealthLevel::Critical
    } else if value >= warning {
        HealthLevel::Warning
    } else {
        HealthLevel::Healthy
    };
    let message = match level {
        HealthLevel::Healthy => format!("{label} is within the healthy range."),
        HealthLevel::Warning => format!("{label} is elevated at {value:.1}%"),
        HealthLevel::Critical => format!("{label} is critically high at {value:.1}%"),
    };
    HealthSignal { kind, level, value, message }
}

fn level_rank(level: &HealthLevel) -> u8 {
    match level {
        HealthLevel::Healthy => 0,
        HealthLevel::Warning => 1,
        HealthLevel::Critical => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monitoring::{MonitorSnapshot, NetworkRate};

    fn snapshot(cpu: f64, memory: f64, swap: f64) -> MonitorSnapshot {
        MonitorSnapshot {
            timestamp_ms: 1,
            cpu_percent: cpu,
            memory_percent: memory,
            swap_percent: swap,
            network: vec![NetworkRate {
                name: "lo".into(),
                rx_bytes_per_second: 0.0,
                tx_bytes_per_second: 0.0,
            }],
            storage_read_bytes_per_second: 0.0,
            storage_write_bytes_per_second: 0.0,
            process_count: 0,
            running_processes: 0,
            baseline: None,
            deviation: None,
        }
    }

    #[test]
    fn healthy_metrics_produce_healthy_status() {
        let result = evaluate(Some(&snapshot(20.0, 40.0, 0.0)), Some(50)).unwrap();
        assert_eq!(result.overall, HealthLevel::Healthy);
    }

    #[test]
    fn elevated_metrics_produce_warning() {
        let result = evaluate(Some(&snapshot(85.0, 40.0, 0.0)), Some(50)).unwrap();
        assert_eq!(result.overall, HealthLevel::Warning);
    }

    #[test]
    fn critical_metrics_produce_critical_status() {
        let result = evaluate(Some(&snapshot(96.0, 40.0, 0.0)), Some(95)).unwrap();
        assert_eq!(result.overall, HealthLevel::Critical);
    }

    #[test]
    fn missing_monitoring_snapshot_is_an_error() {
        assert!(matches!(evaluate(None, None), Err(HealthStatusError::NoSnapshot)));
    }
}