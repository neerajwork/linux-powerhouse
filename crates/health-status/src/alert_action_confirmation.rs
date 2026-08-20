use crate::{AlertEvent, preview_alert_actions};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertActionConfirmation {
    pub action_id: String,
    pub title: String,
    pub confirmed: bool,
    pub authorized: bool,
    pub executable: bool,
    pub requires_privilege: bool,
    pub message: String,
}

/// Record explicit user intent for a known preview without authorizing execution.
///
/// Step 64 establishes the confirmation boundary only. It never mutates system state.
pub fn confirm_alert_action(
    event: &AlertEvent,
    action_id: &str,
) -> Result<AlertActionConfirmation, String> {
    let preview = preview_alert_actions(event)
        .into_iter()
        .find(|item| item.id == action_id)
        .ok_or_else(|| format!("unknown alert action preview: {action_id}"))?;

    Ok(AlertActionConfirmation {
        action_id: preview.id,
        title: preview.title,
        confirmed: true,
        authorized: false,
        executable: false,
        requires_privilege: false,
        message: "Intent confirmed for this bounded action preview. No system changes were authorized or performed.".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlertDecision, AlertEventReason, AlertSeverity, SignalKind};

    fn event() -> AlertEvent {
        AlertEvent {
            timestamp_ms: 1_000,
            performance_timestamp_ms: Some(1_000),
            process_evidence: Vec::new(),
            kind: SignalKind::Cpu,
            severity: AlertSeverity::Warning,
            value: 91.0,
            decision: AlertDecision::Notify,
            reason: AlertEventReason::ActivePolicy,
        }
    }

    #[test]
    fn confirmation_records_intent_without_authorization() {
        let result = confirm_alert_action(&event(), "recheck-condition").expect("known preview");
        assert!(result.confirmed);
        assert!(!result.authorized);
        assert!(!result.executable);
        assert!(!result.requires_privilege);
    }

    #[test]
    fn unknown_preview_is_rejected() {
        let result = confirm_alert_action(&event(), "delete-everything");
        assert!(result.is_err());
    }
}
