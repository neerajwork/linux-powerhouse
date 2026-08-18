use super::{AlertDecision, AlertSeverity, AlertState, HealthLevel, SignalKind};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const DEFAULT_ALERT_HISTORY_LIMIT: usize = 100;
const PROCESS_EVIDENCE_LIMIT: usize = 5;
const PROC: &str = "/proc";
const MEM_INFO: &str = "/proc/meminfo";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertEventReason {
    ActivePolicy,
    Snoozed,
    Dismissed,
    SnoozeExpired,
    CriticalOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertProcessEvidence {
    pub pid: u32,
    pub name: String,
    pub memory_percent: f64,
    pub cpu_time_ticks: u64,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertEvent {
    pub timestamp_ms: u128,
    #[serde(default)]
    pub performance_timestamp_ms: Option<u128>,
    #[serde(default)]
    pub process_evidence: Vec<AlertProcessEvidence>,
    pub kind: SignalKind,
    pub severity: AlertSeverity,
    pub value: f64,
    pub decision: AlertDecision,
    pub reason: AlertEventReason,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertEventHistory {
    limit: usize,
    events: Vec<AlertEvent>,
}

impl Default for AlertEventHistory {
    fn default() -> Self {
        Self::new(DEFAULT_ALERT_HISTORY_LIMIT)
    }
}

impl AlertEventHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            events: Vec::new(),
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn events(&self) -> &[AlertEvent] {
        &self.events
    }

    pub fn record(&mut self, event: AlertEvent) {
        if self.limit == 0 {
            return;
        }
        self.events.push(event);
        let excess = self.events.len().saturating_sub(self.limit);
        if excess > 0 {
            self.events.drain(0..excess);
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

pub fn event_reason(state: AlertState, severity: AlertSeverity, now_ms: u128) -> AlertEventReason {
    if severity == AlertSeverity::Critical {
        return AlertEventReason::CriticalOverride;
    }
    match state {
        AlertState::Active => AlertEventReason::ActivePolicy,
        AlertState::Dismissed => AlertEventReason::Dismissed,
        AlertState::Snoozed { until_ms } if now_ms < until_ms => AlertEventReason::Snoozed,
        AlertState::Snoozed { .. } => AlertEventReason::SnoozeExpired,
    }
}

pub fn create_event(
    timestamp_ms: u128,
    kind: SignalKind,
    level: HealthLevel,
    value: f64,
    state: AlertState,
    decision: AlertDecision,
) -> Option<AlertEvent> {
    let severity = match level {
        HealthLevel::Healthy => return None,
        HealthLevel::Warning => AlertSeverity::Warning,
        HealthLevel::Critical => AlertSeverity::Critical,
    };
    Some(AlertEvent {
        timestamp_ms,
        performance_timestamp_ms: None,
        process_evidence: capture_process_evidence(kind),
        kind,
        severity,
        value,
        decision,
        reason: event_reason(state, severity, timestamp_ms),
    })
}

#[cfg(target_os = "linux")]
fn capture_process_evidence(kind: SignalKind) -> Vec<AlertProcessEvidence> {
    let memory_total = fs::read_to_string(MEM_INFO)
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or(0)
        .saturating_mul(1024);

    let mut entries = Vec::new();
    let Ok(directory) = fs::read_dir(PROC) else {
        return entries;
    };

    for entry in directory.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        let path = entry.path();
        let Ok(status) = fs::read_to_string(path.join("status")) else {
            continue;
        };
        let process_name = status
            .lines()
            .find_map(|line| line.strip_prefix("Name:").map(str::trim))
            .unwrap_or("unknown")
            .to_owned();
        let memory_bytes = status
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0)
            .saturating_mul(1024);
        let cpu_time_ticks = read_cpu_ticks(&path).unwrap_or(0);
        let memory_percent = if memory_total == 0 {
            0.0
        } else {
            100.0 * memory_bytes as f64 / memory_total as f64
        };
        entries.push(AlertProcessEvidence {
            pid,
            name: process_name,
            memory_percent,
            cpu_time_ticks,
            rank: 0,
        });
        if entries.len() >= 500 {
            break;
        }
    }

    match kind {
        SignalKind::Memory => entries.sort_by(|a, b| {
            b.memory_percent
                .total_cmp(&a.memory_percent)
                .then_with(|| a.pid.cmp(&b.pid))
        }),
        SignalKind::Cpu => entries.sort_by(|a, b| {
            b.cpu_time_ticks
                .cmp(&a.cpu_time_ticks)
                .then_with(|| a.pid.cmp(&b.pid))
        }),
        _ => return Vec::new(),
    }

    entries.truncate(PROCESS_EVIDENCE_LIMIT);
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.rank = index + 1;
    }
    entries
}

#[cfg(not(target_os = "linux"))]
fn capture_process_evidence(_kind: SignalKind) -> Vec<AlertProcessEvidence> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn read_cpu_ticks(path: &Path) -> Option<u64> {
    let stat = fs::read_to_string(path.join("stat")).ok()?;
    let (_, fields) = stat.split_once(')')?;
    let fields: Vec<&str> = fields.split_whitespace().collect();
    if fields.len() < 13 {
        return None;
    }
    let utime = fields[11].parse::<u64>().ok()?;
    let stime = fields[12].parse::<u64>().ok()?;
    Some(utime.saturating_add(stime))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(timestamp_ms: u128) -> AlertEvent {
        AlertEvent {
            timestamp_ms,
            performance_timestamp_ms: None,
            process_evidence: Vec::new(),
            kind: SignalKind::Cpu,
            severity: AlertSeverity::Warning,
            value: 85.0,
            decision: AlertDecision::Notify,
            reason: AlertEventReason::ActivePolicy,
        }
    }

    #[test]
    fn history_keeps_only_the_newest_events() {
        let mut history = AlertEventHistory::new(2);
        for timestamp_ms in 1..=3 {
            history.record(event(timestamp_ms));
        }
        assert_eq!(history.events().len(), 2);
        assert_eq!(history.events()[0].timestamp_ms, 2);
        assert_eq!(history.events()[1].timestamp_ms, 3);
    }

    #[test]
    fn zero_limit_does_not_retain_events() {
        let mut history = AlertEventHistory::new(0);
        history.record(event(1));
        assert!(history.events().is_empty());
    }

    #[test]
    fn clear_removes_all_events() {
        let mut history = AlertEventHistory::new(10);
        history.record(event(1));
        history.clear();
        assert!(history.events().is_empty());
    }

    #[test]
    fn critical_alert_records_override_reason() {
        let event = create_event(
            1_000,
            SignalKind::Cpu,
            HealthLevel::Critical,
            96.0,
            AlertState::Snoozed { until_ms: 10_000 },
            AlertDecision::Notify,
        )
        .unwrap();
        assert_eq!(event.reason, AlertEventReason::CriticalOverride);
    }

    #[test]
    fn active_warning_records_notify_reason() {
        let event = create_event(
            1_000,
            SignalKind::Memory,
            HealthLevel::Warning,
            85.0,
            AlertState::Active,
            AlertDecision::Notify,
        )
        .unwrap();
        assert_eq!(event.reason, AlertEventReason::ActivePolicy);
    }

    #[test]
    fn snoozed_warning_records_suppressed_reason() {
        let event = create_event(
            1_000,
            SignalKind::Swap,
            HealthLevel::Warning,
            25.0,
            AlertState::Snoozed { until_ms: 2_000 },
            AlertDecision::Suppressed,
        )
        .unwrap();
        assert_eq!(event.reason, AlertEventReason::Snoozed);
    }

    #[test]
    fn expired_snooze_records_expiry_reason() {
        let event = create_event(
            2_000,
            SignalKind::Storage,
            HealthLevel::Warning,
            82.0,
            AlertState::Snoozed { until_ms: 1_000 },
            AlertDecision::Notify,
        )
        .unwrap();
        assert_eq!(event.reason, AlertEventReason::SnoozeExpired);
    }

    #[test]
    fn healthy_signal_has_no_event() {
        assert!(
            create_event(
                1_000,
                SignalKind::Network,
                HealthLevel::Healthy,
                20.0,
                AlertState::Active,
                AlertDecision::Notify
            )
            .is_none()
        );
    }

    #[test]
    fn process_evidence_is_bounded() {
        let event = create_event(
            1_000,
            SignalKind::Memory,
            HealthLevel::Warning,
            85.0,
            AlertState::Active,
            AlertDecision::Notify,
        )
        .unwrap();
        assert!(event.process_evidence.len() <= PROCESS_EVIDENCE_LIMIT);
        assert!(
            event
                .process_evidence
                .windows(2)
                .all(|pair| pair[0].rank < pair[1].rank)
        );
    }
}
