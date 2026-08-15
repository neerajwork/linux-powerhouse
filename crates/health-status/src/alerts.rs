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
    pub severity: AlertSeverity,
    pub state: AlertState,
}

impl AlertPolicy {
    pub fn new(severity: AlertSeverity) -> Self {
        Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_warning_is_notified() {
        let policy = AlertPolicy::new(AlertSeverity::Warning);

        assert_eq!(policy.decision(1_000), AlertDecision::Notify);
    }

    #[test]
    fn dismissed_warning_is_suppressed() {
        let mut policy = AlertPolicy::new(AlertSeverity::Warning);
        policy.dismiss();

        assert_eq!(policy.decision(1_000), AlertDecision::Suppressed);
    }

    #[test]
    fn warning_stays_suppressed_until_snooze_expires() {
        let mut policy = AlertPolicy::new(AlertSeverity::Warning);
        policy.snooze_until(2_000);

        assert_eq!(policy.decision(1_999), AlertDecision::Suppressed);
        assert_eq!(policy.decision(2_000), AlertDecision::Notify);
    }

    #[test]
    fn restored_warning_is_notified_again() {
        let mut policy = AlertPolicy::new(AlertSeverity::Warning);
        policy.snooze_until(2_000);
        policy.restore();

        assert_eq!(policy.decision(1_000), AlertDecision::Notify);
    }

    #[test]
    fn critical_alert_cannot_be_suppressed() {
        let mut policy = AlertPolicy::new(AlertSeverity::Critical);

        policy.dismiss();
        assert_eq!(policy.decision(1_000), AlertDecision::Notify);

        policy.snooze_until(10_000);
        assert_eq!(policy.decision(1_000), AlertDecision::Notify);
    }
}
