use super::SignalKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertState {
    Active,
    Dismissed,
    Snoozed { until_ms: u128 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertDecision {
    Notify,
    Suppressed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlertPolicy {
    pub kind: SignalKind,
    pub severity: AlertSeverity,
    pub state: AlertState,
}

impl AlertPolicy {
    pub fn new(kind: SignalKind, severity: AlertSeverity) -> Self {
        Self {
            kind,
            severity,
            state: AlertState::Active,
        }
    }

    pub fn dismiss(&mut self) {
        self.state = AlertState::Dismissed;
    }

    pub fn snooze_until(&mut self, until_ms: u128) {
        self.state = AlertState::Snoozed { until_ms };
    }

    pub fn restore(&mut self) {
        self.state = AlertState::Active;
    }

    pub fn decision(&self, now_ms: u128) -> AlertDecision {
        if self.severity == AlertSeverity::Critical {
            return AlertDecision::Notify;
        }

        match self.state {
            AlertState::Active => AlertDecision::Notify,
            AlertState::Dismissed => AlertDecision::Suppressed,
            AlertState::Snoozed { until_ms } if now_ms < until_ms => AlertDecision::Suppressed,
            AlertState::Snoozed { .. } => AlertDecision::Notify,
        }
    }
}
pub fn alert_decision(
    kind: SignalKind,
    level: super::HealthLevel,
    state: AlertState,
    now_ms: u128,
) -> Option<AlertDecision> {
    let severity = match level {
        super::HealthLevel::Healthy => return None,
        super::HealthLevel::Warning => AlertSeverity::Warning,
        super::HealthLevel::Critical => AlertSeverity::Critical,
    };

    let policy = AlertPolicy {
        kind,
        severity,
        state,
    };

    Some(policy.decision(now_ms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HealthLevel;

    #[test]
    fn active_warning_is_notified() {
        let policy = AlertPolicy::new(SignalKind::Cpu, AlertSeverity::Warning);

        assert_eq!(policy.decision(1_000), AlertDecision::Notify);
    }

    #[test]
    fn dismissed_warning_is_suppressed() {
        let mut policy = AlertPolicy::new(SignalKind::Cpu, AlertSeverity::Warning);
        policy.dismiss();

        assert_eq!(policy.decision(1_000), AlertDecision::Suppressed);
    }

    #[test]
    fn warning_stays_suppressed_until_snooze_expires() {
        let mut policy = AlertPolicy::new(SignalKind::Cpu, AlertSeverity::Warning);
        policy.snooze_until(2_000);

        assert_eq!(policy.decision(1_999), AlertDecision::Suppressed);
        assert_eq!(policy.decision(2_000), AlertDecision::Notify);
    }

    #[test]
    fn restored_warning_is_notified_again() {
        let mut policy = AlertPolicy::new(SignalKind::Cpu, AlertSeverity::Warning);
        policy.snooze_until(2_000);
        policy.restore();

        assert_eq!(policy.decision(1_000), AlertDecision::Notify);
    }

    #[test]
    fn critical_alert_cannot_be_suppressed() {
        let mut policy = AlertPolicy::new(SignalKind::Cpu, AlertSeverity::Critical);

        policy.dismiss();
        assert_eq!(policy.decision(1_000), AlertDecision::Notify);

        policy.snooze_until(10_000);
        assert_eq!(policy.decision(1_000), AlertDecision::Notify);
    }

    #[test]
    fn healthy_signal_has_no_alert() {
        assert_eq!(
            alert_decision(
                SignalKind::Cpu,
                HealthLevel::Healthy,
                AlertState::Active,
                1_000,
            ),
            None
        );
    }

    #[test]
    fn warning_signal_respects_snooze() {
        assert_eq!(
            alert_decision(
                SignalKind::Memory,
                HealthLevel::Warning,
                AlertState::Snoozed { until_ms: 2_000 },
                1_000,
            ),
            Some(AlertDecision::Suppressed)
        );
    }

    #[test]
    fn critical_signal_ignores_snooze() {
        assert_eq!(
            alert_decision(
                SignalKind::Storage,
                HealthLevel::Critical,
                AlertState::Snoozed { until_ms: 10_000 },
                1_000,
            ),
            Some(AlertDecision::Notify)
        );
    }

    #[test]
    fn critical_signal_ignores_dismissal() {
        assert_eq!(
            alert_decision(
                SignalKind::Network,
                HealthLevel::Critical,
                AlertState::Dismissed,
                1_000,
            ),
            Some(AlertDecision::Notify)
        );
    }
}
