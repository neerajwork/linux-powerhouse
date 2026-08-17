use super::{AlertDecision, AlertSeverity, AlertState, HealthLevel, SignalKind};
use serde::{Deserialize, Serialize};

pub const DEFAULT_ALERT_HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertEventReason {
    ActivePolicy,
    Snoozed,
    Dismissed,
    SnoozeExpired,
    CriticalOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertEvent {
    pub timestamp_ms: u128,
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
        kind,
        severity,
        value,
        decision,
        reason: event_reason(state, severity, timestamp_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_keeps_only_the_newest_events() {
        let mut history = AlertEventHistory::new(2);

        for timestamp_ms in 1..=3 {
            history.record(AlertEvent {
                timestamp_ms,
                kind: SignalKind::Cpu,
                severity: AlertSeverity::Warning,
                value: 85.0,
                decision: AlertDecision::Notify,
                reason: AlertEventReason::ActivePolicy,
            });
        }

        assert_eq!(history.events().len(), 2);
        assert_eq!(history.events()[0].timestamp_ms, 2);
        assert_eq!(history.events()[1].timestamp_ms, 3);
    }

    #[test]
    fn zero_limit_does_not_retain_events() {
        let mut history = AlertEventHistory::new(0);
        history.record(AlertEvent {
            timestamp_ms: 1,
            kind: SignalKind::Memory,
            severity: AlertSeverity::Warning,
            value: 85.0,
            decision: AlertDecision::Suppressed,
            reason: AlertEventReason::Snoozed,
        });

        assert!(history.events().is_empty());
    }

    #[test]
    fn clear_removes_all_events() {
        let mut history = AlertEventHistory::new(10);
        history.record(AlertEvent {
            timestamp_ms: 1,
            kind: SignalKind::Storage,
            severity: AlertSeverity::Warning,
            value: 82.0,
            decision: AlertDecision::Notify,
            reason: AlertEventReason::ActivePolicy,
        });

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
                AlertDecision::Notify,
            )
            .is_none()
        );
    }
}
