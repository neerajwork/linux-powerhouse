use crate::{AlertEvent, AlertEventReason, AlertSeverity, SignalKind};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertGuidance {
    pub summary: String,
    pub steps: Vec<String>,
    pub safety_note: String,
}

/// Produce deterministic, non-mutating guidance for an alert.
///
/// Guidance is intentionally limited to investigation and user-controlled actions.
/// It never executes, authorizes, or requests privileged remediation.
pub fn guide_alert(event: &AlertEvent) -> AlertGuidance {
    let summary = match event.kind {
        SignalKind::Cpu => "Review the processes contributing to sustained CPU load.".to_owned(),
        SignalKind::Memory => {
            "Review the processes contributing to elevated memory usage.".to_owned()
        }
        SignalKind::Swap => "Check memory pressure and whether swap usage remains elevated.".to_owned(),
        SignalKind::Storage => {
            "Check filesystem usage and recent disk activity before taking action.".to_owned()
        }
        SignalKind::Network => {
            "Review recent network activity and connectivity if the condition persists.".to_owned()
        }
    };

    let mut steps = match event.kind {
        SignalKind::Cpu => vec![
            "Review the recorded process contributors and their CPU activity.".to_owned(),
            "Check whether the elevated CPU condition persists across subsequent samples.".to_owned(),
            "If a process is no longer needed, close it through its normal application controls.".to_owned(),
        ],
        SignalKind::Memory => vec![
            "Review the recorded process contributors and their memory usage.".to_owned(),
            "Check whether memory usage remains elevated over time.".to_owned(),
            "If an application is no longer needed, close it through its normal application controls.".to_owned(),
        ],
        SignalKind::Swap => vec![
            "Review memory usage and the processes using the most memory.".to_owned(),
            "Check whether swap usage remains elevated after memory pressure changes.".to_owned(),
            "Avoid forcing processes to stop unless you have independently confirmed that action is appropriate.".to_owned(),
        ],
        SignalKind::Storage => vec![
            "Review filesystem utilization and recent storage activity.".to_owned(),
            "Identify large or unnecessary files using a trusted storage tool.".to_owned(),
            "Confirm backups or other safeguards before deleting or modifying data.".to_owned(),
        ],
        SignalKind::Network => vec![
            "Check whether the network condition persists.".to_owned(),
            "Review the affected connection, interface, or service using trusted diagnostics.".to_owned(),
            "Avoid changing network configuration until the affected component is understood.".to_owned(),
        ],
    };

    if event.severity == AlertSeverity::Critical {
        steps.insert(
            0,
            "Treat the event as time-sensitive and verify the underlying condition before making changes.".to_owned(),
        );
    }

    if event.reason == AlertEventReason::Snoozed || event.reason == AlertEventReason::Dismissed {
        steps.push(
            "The warning was previously suppressed or dismissed; review it again if the condition persists."
                .to_owned(),
        );
    }

    AlertGuidance {
        summary,
        steps,
        safety_note: "Guidance is informational only. No system changes are performed, and these observations do not prove causation.".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlertDecision, AlertSeverity};

    fn event(kind: SignalKind, severity: AlertSeverity, reason: AlertEventReason) -> AlertEvent {
        AlertEvent {
            timestamp_ms: 1_000,
            performance_timestamp_ms: Some(1_000),
            process_evidence: Vec::new(),
            kind,
            severity,
            value: 91.0,
            decision: AlertDecision::Notify,
            reason,
        }
    }

    #[test]
    fn memory_guidance_is_investigation_only() {
        let guidance = guide_alert(&event(
            SignalKind::Memory,
            AlertSeverity::Warning,
            AlertEventReason::ActivePolicy,
        ));

        assert!(guidance.summary.contains("memory"));
        assert!(guidance.steps.iter().any(|step| step.contains("process")));
        assert!(guidance.safety_note.contains("No system changes"));
    }

    #[test]
    fn critical_guidance_prioritizes_verification() {
        let guidance = guide_alert(&event(
            SignalKind::Cpu,
            AlertSeverity::Critical,
            AlertEventReason::CriticalOverride,
        ));

        assert!(guidance.steps[0].contains("time-sensitive"));
        assert!(guidance.safety_note.contains("do not prove causation"));
    }

    #[test]
    fn dismissed_warning_reminds_user_to_recheck_persistence() {
        let guidance = guide_alert(&event(
            SignalKind::Storage,
            AlertSeverity::Warning,
            AlertEventReason::Dismissed,
        ));

        assert!(guidance
            .steps
            .iter()
            .any(|step| step.contains("previously suppressed or dismissed")));
    }
}
