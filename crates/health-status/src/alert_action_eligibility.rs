use crate::{AlertActionConfirmation, AlertActionPreview};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertActionEligibility {
    pub action_id: String,
    pub confirmed: bool,
    pub authorized: bool,
    pub executable: bool,
    pub eligible: bool,
    pub requires_privilege: bool,
    pub message: String,
}

/// Determine whether a confirmed action satisfies the execution boundary.
///
/// Step 65 defines the safety contract only. No action is authorized or executed here.
pub fn evaluate_action_eligibility(
    preview: &AlertActionPreview,
    confirmation: &AlertActionConfirmation,
) -> AlertActionEligibility {
    let matches_action = preview.id == confirmation.action_id;
    let eligible = matches_action
        && confirmation.confirmed
        && confirmation.authorized
        && confirmation.executable
        && !preview.requires_privilege;

    let message = if !matches_action {
        "Action preview and confirmation do not refer to the same action.".to_owned()
    } else if !confirmation.confirmed {
        "Explicit user confirmation is required before execution can be considered.".to_owned()
    } else if !confirmation.authorized {
        "The action is confirmed but not authorized for execution.".to_owned()
    } else if !confirmation.executable || preview.executable {
        "The action is not executable under the current safety contract.".to_owned()
    } else if preview.requires_privilege || confirmation.requires_privilege {
        "Privileged actions are not eligible under the Step 65 execution boundary.".to_owned()
    } else {
        "Action satisfies the Step 65 eligibility contract; execution remains a separate boundary.".to_owned()
    };

    AlertActionEligibility {
        action_id: confirmation.action_id.clone(),
        confirmed: confirmation.confirmed,
        authorized: confirmation.authorized,
        executable: confirmation.executable,
        eligible,
        requires_privilege: preview.requires_privilege || confirmation.requires_privilege,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preview() -> AlertActionPreview {
        AlertActionPreview {
            id: "recheck-condition".to_owned(),
            title: "Recheck condition".to_owned(),
            reason: "Verify persistence".to_owned(),
            scope: "Current observation".to_owned(),
            safety: "Preview only".to_owned(),
            requires_privilege: false,
            executable: false,
        }
    }

    fn confirmation(confirmed: bool, authorized: bool, executable: bool) -> AlertActionConfirmation {
        AlertActionConfirmation {
            action_id: "recheck-condition".to_owned(),
            title: "Recheck condition".to_owned(),
            confirmed,
            authorized,
            executable,
            requires_privilege: false,
            message: String::new(),
        }
    }

    #[test]
    fn current_confirmation_is_not_execution_eligible() {
        let result = evaluate_action_eligibility(&preview(), &confirmation(true, false, false));
        assert!(!result.eligible);
        assert!(!result.authorized);
        assert!(!result.executable);
    }

    #[test]
    fn mismatched_action_is_rejected() {
        let mut confirmation = confirmation(true, true, true);
        confirmation.action_id = "unknown-action".to_owned();
        let result = evaluate_action_eligibility(&preview(), &confirmation);
        assert!(!result.eligible);
    }

    #[test]
    fn privilege_is_never_eligible() {
        let mut preview = preview();
        preview.requires_privilege = true;
        let confirmation = confirmation(true, true, true);
        let result = evaluate_action_eligibility(&preview, &confirmation);
        assert!(!result.eligible);
        assert!(result.requires_privilege);
    }

    #[test]
    fn eligibility_does_not_execute() {
        let confirmation = confirmation(true, true, true);
        let result = evaluate_action_eligibility(&preview(), &confirmation);
        assert!(result.eligible);
        assert!(result.message.contains("separate boundary"));
    }
}
