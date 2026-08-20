use crate::{AlertEvent, AlertEventReason, AlertSeverity, SignalKind};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertActionPreview {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub scope: String,
    pub safety: String,
    pub requires_privilege: bool,
    pub executable: bool,
}

/// Describe bounded, user-controlled actions that could be considered for an alert.
///
/// Step 63 is preview-only: no action is executable, authorized, or privileged.
pub fn preview_alert_actions(event: &AlertEvent) -> Vec<AlertActionPreview> {
    let mut previews = match event.kind {
        SignalKind::Cpu => vec![
            preview(
                "review-processes",
                "Review process contributors",
                "Inspect the bounded process evidence already recorded for this alert.",
                "Current alert context only",
            ),
            preview(
                "recheck-condition",
                "Recheck the CPU condition",
                "Verify whether elevated CPU usage persists across subsequent monitoring samples.",
                "Current system observation",
            ),
        ],
        SignalKind::Memory => vec![
            preview(
                "review-processes",
                "Review process contributors",
                "Inspect the bounded process evidence already recorded for this alert.",
                "Current alert context only",
            ),
            preview(
                "recheck-condition",
                "Recheck the memory condition",
                "Verify whether elevated memory usage persists across subsequent monitoring samples.",
                "Current system observation",
            ),
        ],
        SignalKind::Swap => vec![
            preview(
                "review-memory-pressure",
                "Review memory pressure",
                "Inspect memory and swap usage before considering any user-controlled change.",
                "Current system observation",
            ),
            preview(
                "recheck-condition",
                "Recheck the swap condition",
                "Verify whether elevated swap usage persists after memory pressure changes.",
                "Current system observation",
            ),
        ],
        SignalKind::Storage => vec![
            preview(
                "review-storage",
                "Review storage usage",
                "Inspect filesystem utilization and recent disk activity before changing data.",
                "Filesystem observation",
            ),
            preview(
                "verify-backups",
                "Verify data safeguards",
                "Confirm backups or other safeguards before considering deletion or modification.",
                "User-controlled data review",
            ),
        ],
        SignalKind::Network => vec![
            preview(
                "review-network",
                "Review network diagnostics",
                "Inspect the affected connection, interface, or service using trusted diagnostics.",
                "Affected network context",
            ),
            preview(
                "recheck-condition",
                "Recheck the network condition",
                "Verify whether the connectivity or activity condition persists.",
                "Current system observation",
            ),
        ],
    };

    if event.severity == AlertSeverity::Critical {
        previews.insert(
            0,
            preview(
                "verify-critical-condition",
                "Verify the critical condition",
                "Confirm the underlying condition before considering any change.",
                "Critical alert context",
            ),
        );
    }

    if matches!(
        event.reason,
        AlertEventReason::Snoozed | AlertEventReason::Dismissed
    ) {
        previews.push(preview(
            "recheck-suppressed-warning",
            "Recheck the suppressed warning",
            "Review the condition again if it persists after the warning was snoozed or dismissed.",
            "Previously suppressed alert",
        ));
    }

    previews
}

fn preview(id: &str, title: &str, reason: &str, scope: &str) -> AlertActionPreview {
    AlertActionPreview {
        id: id.to_owned(),
        title: title.to_owned(),
        reason: reason.to_owned(),
        scope: scope.to_owned(),
        safety: "Preview only — no system changes are performed.".to_owned(),
        requires_privilege: false,
        executable: false,
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
    fn previews_are_never_executable() {
        for kind in [
            SignalKind::Cpu,
            SignalKind::Memory,
            SignalKind::Swap,
            SignalKind::Storage,
            SignalKind::Network,
        ] {
            let previews = preview_alert_actions(&event(
                kind,
                AlertSeverity::Warning,
                AlertEventReason::ActivePolicy,
            ));
            assert!(!previews.is_empty());
            assert!(
                previews
                    .iter()
                    .all(|item| !item.executable && !item.requires_privilege)
            );
        }
    }

    #[test]
    fn critical_alerts_start_with_verification() {
        let previews = preview_alert_actions(&event(
            SignalKind::Cpu,
            AlertSeverity::Critical,
            AlertEventReason::CriticalOverride,
        ));
        assert_eq!(previews[0].id, "verify-critical-condition");
    }

    #[test]
    fn dismissed_alerts_get_recheck_preview() {
        let previews = preview_alert_actions(&event(
            SignalKind::Storage,
            AlertSeverity::Warning,
            AlertEventReason::Dismissed,
        ));
        assert!(
            previews
                .iter()
                .any(|item| item.id == "recheck-suppressed-warning")
        );
    }
}
