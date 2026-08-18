use crate::{AlertEvent, SignalKind};
use monitoring::MonitorSnapshot;
use serde::{Deserialize, Serialize};

const CORRELATION_WINDOW_MS: u128 = 30_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertPerformanceCorrelation {
    pub alert_timestamp_ms: u128,
    pub snapshot_timestamp_ms: u128,
    pub age_ms: u128,
    pub signal: SignalKind,
    pub alert_value: f64,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub swap_percent: f64,
    pub storage_read_bytes_per_second: f64,
    pub storage_write_bytes_per_second: f64,
    pub process_count: usize,
    pub running_processes: usize,
    pub primary_evidence: String,
}

pub fn correlate_alert(
    event: &AlertEvent,
    snapshots: &[MonitorSnapshot],
) -> Option<AlertPerformanceCorrelation> {
    let performance_timestamp = event.performance_timestamp_ms?;
    let snapshot = snapshots
        .iter()
        .filter_map(|snapshot| {
            let age_ms = snapshot.timestamp_ms.abs_diff(performance_timestamp);
            (age_ms <= CORRELATION_WINDOW_MS).then_some((age_ms, snapshot))
        })
        .min_by_key(|(age_ms, _)| *age_ms)
        .map(|(_, snapshot)| snapshot)?;

    Some(AlertPerformanceCorrelation {
        alert_timestamp_ms: event.timestamp_ms,
        snapshot_timestamp_ms: snapshot.timestamp_ms,
        age_ms: snapshot.timestamp_ms.abs_diff(performance_timestamp),
        signal: event.kind,
        alert_value: event.value,
        cpu_percent: snapshot.cpu_percent,
        memory_percent: snapshot.memory_percent,
        swap_percent: snapshot.swap_percent,
        storage_read_bytes_per_second: snapshot.storage_read_bytes_per_second,
        storage_write_bytes_per_second: snapshot.storage_write_bytes_per_second,
        process_count: snapshot.process_count,
        running_processes: snapshot.running_processes,
        primary_evidence: primary_evidence(event.kind, snapshot),
    })
}

fn primary_evidence(signal: SignalKind, snapshot: &MonitorSnapshot) -> String {
    match signal {
        SignalKind::Cpu => format!(
            "CPU utilization was {:.1}% near the alert.",
            snapshot.cpu_percent
        ),
        SignalKind::Memory => format!(
            "Memory utilization was {:.1}% near the alert.",
            snapshot.memory_percent
        ),
        SignalKind::Swap => format!(
            "Swap utilization was {:.1}% near the alert.",
            snapshot.swap_percent
        ),
        SignalKind::Storage => format!(
            "Storage I/O was {:.1} read / {:.1} write bytes/s near the alert.",
            snapshot.storage_read_bytes_per_second, snapshot.storage_write_bytes_per_second
        ),
        SignalKind::Network => format!(
            "Network activity was sampled across {} interfaces near the alert.",
            snapshot.network.len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlertDecision, AlertEventReason, AlertSeverity};

    fn snapshot(timestamp_ms: u128) -> MonitorSnapshot {
        MonitorSnapshot {
            timestamp_ms,
            cpu_percent: 91.0,
            memory_percent: 78.0,
            swap_percent: 12.0,
            network: Vec::new(),
            storage_read_bytes_per_second: 1_000.0,
            storage_write_bytes_per_second: 2_000.0,
            process_count: 100,
            running_processes: 8,
            baseline: None,
            deviation: None,
        }
    }

    fn event(timestamp_ms: u128, performance_timestamp_ms: Option<u128>) -> AlertEvent {
        AlertEvent {
            timestamp_ms,
            performance_timestamp_ms,
            kind: SignalKind::Cpu,
            severity: AlertSeverity::Warning,
            value: 90.0,
            decision: AlertDecision::Notify,
            reason: AlertEventReason::ActivePolicy,
            process_evidence: Vec::new(),
        }
    }

    #[test]
    fn selects_nearest_snapshot_within_window() {
        let result = correlate_alert(
            &event(100_000, Some(10_000)),
            &[snapshot(9_000), snapshot(10_100)],
        )
        .unwrap();
        assert_eq!(result.snapshot_timestamp_ms, 10_100);
        assert_eq!(result.age_ms, 100);
        assert_eq!(result.signal, SignalKind::Cpu);
        assert!(result.primary_evidence.contains("91.0%"));
    }

    #[test]
    fn ignores_snapshots_outside_window() {
        assert!(correlate_alert(&event(100_000, Some(10_000)), &[snapshot(40_001)]).is_none());
    }

    #[test]
    fn requires_performance_timestamp() {
        assert!(correlate_alert(&event(100_000, None), &[snapshot(10_000)]).is_none());
    }
}
