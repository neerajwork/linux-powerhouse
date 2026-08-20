use crate::AlertActionConfirmation;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertActionAuthorization {
    pub action_id: String,
    pub authorized: bool,
    pub requires_privilege: bool,
    pub message: String,
}

/// Record explicit authorization for one already-confirmed action.
///
/// Authorization is a separate boundary from confirmation and execution. It
/// never performs the action and never changes the action's scope or privilege
/// requirements.
pub fn authorize_alert_action(
    confirmation: &AlertActionConfirmation,
    action_id: &str,
) -> Result<AlertActionAuthorization, String> {
    if confirmation.action_id != action_id {
        return Err(format!(
            "authorization action does not match confirmation: {action_id}"
        ));
    }

    if !confirmation.confirmed {
        return Err("explicit confirmation is required before authorization".to_owned());
    }

    if confirmation.requires_privilege {
        return Err(
            "privileged actions cannot be authorized on the current non-privileged path".to_owned(),
        );
    }

    Ok(AlertActionAuthorization {
        action_id: confirmation.action_id.clone(),
        authorized: true,
        requires_privilege: confirmation.requires_privilege,
        message: "Action authorization recorded. No system changes were performed.".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirmation(confirmed: bool) -> AlertActionConfirmation {
        AlertActionConfirmation {
            action_id: "recheck-condition".to_owned(),
            title: "Recheck condition".to_owned(),
            confirmed,
            authorized: false,
            executable: false,
            requires_privilege: false,
            message: String::new(),
        }
    }

    #[test]
    fn matching_confirmed_action_can_be_authorized_without_execution() {
        let result = authorize_alert_action(&confirmation(true), "recheck-condition")
            .expect("matching confirmed action");
        assert!(result.authorized);
        assert!(!result.requires_privilege);
    }

    #[test]
    fn authorization_requires_matching_action_id() {
        let result = authorize_alert_action(&confirmation(true), "delete-everything");
        assert!(result.is_err());
    }

    #[test]
    fn authorization_requires_confirmation() {
        let result = authorize_alert_action(&confirmation(false), "recheck-condition");
        assert!(result.is_err());
    }

    #[test]
    fn privileged_action_cannot_be_authorized() {
        let mut value = confirmation(true);
        value.requires_privilege = true;
        let result = authorize_alert_action(&value, "recheck-condition");
        assert!(result.is_err());
    }
}
