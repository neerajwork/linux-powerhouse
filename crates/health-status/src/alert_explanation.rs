use crate::{AlertDecision, AlertEvent, AlertEventReason, AlertPerformanceCorrelation, AlertSeverity, SignalKind};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertExplanation {
    pub headline: String,
    pub detail: String,
    pub evidence: String,
    pub action: String,
}

pub fn explain_alert(
    event: &AlertEvent,
    correlation: Option<&AlertPerformanceCorrelation>,
) -> AlertExplanation {
    let headline = match event.severity {
        AlertSeverity::Critical => format!("Critical {} alert", signal_label(event.kind)),
        AlertSeverity::Warning => format!("{} warning alert", signal_label(event.kind)),
    };

    let detail = match event.decision {
        AlertDecision::Notify => format!(
            "The {} signal reached {:.1}%, so the active alert policy notified you.",
            signal_label(event.kind),
            event.value
        ),
        AlertDecision::Suppressed => format!(
            "The {} signal reached {:.1}%, but the routine warning was suppressed by the current alert policy.",
            signal_label(event.kind),
            event.value
        ),
    };

    let evidence = correlation
        .map(|item| item.primary_evidence.clone())
        .unwrap_or_else(|| "No matching performance snapshot was available for this alert.".to_owned());

    let action = match event.reason {
        AlertEventReason::CriticalOverride =>
            "Critical events remain visible regardless of routine warning preferences.".to_owned(),
        AlertEventReason::Snoozed =>
            "The routine warning was snoozed; review the event if the underlying condition persists.".to_owned(),
        AlertEventReason::Dismissed =>
            "The routine warning was dismissed; review the history if the condition returns.".to_owned(),
        AlertEventReason::SnoozeExpired =>
            "The snooze period had expired, so the routine warning returned to the active policy.".to_owned(),
        AlertEventReason::ActivePolicy =>
            "The active alert policy determined that this event should be recorded and handled according to its decision.".to_owned(),
    };

    AlertExplanation {
        headline,
        detail,
        evidence,
        action,
    }
}

fn signal_label(signal: SignalKind) -> &'static str {
    match signal {
        SignalKind::Cpu => "CPU",
        SignalKind::Memory => "memory",
        SignalKind::Swap => "swap",
        SignalKind::Storage => "storage",
        SignalKind::Network => "network",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlertEventReason, AlertSeverity};

    fn event(decision: AlertDecision, reason: AlertEventReason) -> AlertEvent {
        AlertEvent {
            timestamp_ms: 1_000,
            performance_timestamp_ms: Some(1_000),
            kind: SignalKind::Cpu,
            severity: AlertSeverity::Warning,
            value: 91.5,
            decision,
            reason,
        }
    }

    #[test]
    fn explains_notified_alert_with_performance_evidence() {
        let correlation = AlertPerformanceCorrelation {
            alert_timestamp_ms: 1_000,
            snapshot_timestamp_ms: 1_100,
            age_ms: 100,
            signal: SignalKind::Cpu,
            alert_value: 91.5,
            cpu_percent: 93.0,
            memory_percent: 72.0,
            swap_percent: 5.0,
            storage_read_bytes_per_second: 10.0,
            storage_write_bytes_per_second: 20.0,
            process_count: 100,
            running_processes: 10,
            primary_evidence: "CPU utilization was 93.0% near the alert.".to_owned(),
        };

        let explanation = explain_alert(
            &event(AlertDecision::Notify, AlertEventReason::ActivePolicy),
            Some(&correlation),
        );

        assert_eq!(explanation.headline, "CPU warning alert");
        assert!(explanation.detail.contains("91.5%"));
        assert!(explanation.evidence.contains("93.0%"));
    }

    #[test]
    fn explains_suppressed_alert_without_claiming_notification() {
        let explanation = explain_alert(
            &event(AlertDecision::Suppressed, AlertEventReason::Snoozed),
            None,
        );

        assert!(explanation.detail.contains("suppressed"));
        assert!(explanation.action.contains("snoozed"));
        assert!(explanation.evidence.contains("No matching"));
    }

    #[test]
    fn critical_explanation_preserves_override_safety_message() {
        let mut critical = event(AlertDecision::Notify, AlertEventReason::CriticalOverride);
        critical.severity = AlertSeverity::Critical;
        let explanation = explain_alert(&critical, None);

        assert_eq!(explanation.headline, "Critical CPU alert");
        assert!(explanation.action.contains("remain visible"));
    }
}
