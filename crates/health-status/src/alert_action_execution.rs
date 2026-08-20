use crate::{AlertActionAuthorization, AlertActionConfirmation};

const RECHECK_CONDITION_ACTION_ID: &str = "recheck-condition";

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct AlertActionExecutionEligibility {
    pub action_id: String,
    pub eligible: bool,
    pub authorized: bool,
    pub executable: bool,
    pub requires_privilege: bool,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct AlertActionExecutionRequest {
    pub action_id: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct AlertActionExecutionResult {
    pub action_id: String,
    pub executed: bool,
    pub message: String,
}

/// Establish the execution boundary after explicit confirmation.
pub fn evaluate_execution_eligibility(
    confirmation: &AlertActionConfirmation,
) -> AlertActionExecutionEligibility {
    let eligible = confirmation.confirmed
        && confirmation.authorized
        && confirmation.executable
        && !confirmation.requires_privilege;

    AlertActionExecutionEligibility {
        action_id: confirmation.action_id.clone(),
        eligible,
        authorized: confirmation.authorized,
        executable: confirmation.executable,
        requires_privilege: confirmation.requires_privilege,
        message: if eligible {
            "Action satisfies the current execution eligibility checks.".to_owned()
        } else {
            "Execution is not eligible: explicit confirmation alone does not authorize or enable execution."
                .to_owned()
        },
    }
}

/// Create a request for a known, explicitly allowlisted action.
///
/// The request contains only the action identifier. It never accepts an
/// arbitrary command, shell expression, path, or executable from the caller.
pub fn create_execution_request(
    confirmation: &AlertActionConfirmation,
    authorization: &AlertActionAuthorization,
) -> Result<AlertActionExecutionRequest, String> {
    if !confirmation.confirmed {
        return Err("execution requires explicit confirmation".to_owned());
    }

    if !authorization.authorized {
        return Err("execution requires explicit action authorization".to_owned());
    }

    if confirmation.action_id != authorization.action_id {
        return Err("authorization does not match the confirmed action".to_owned());
    }

    if confirmation.requires_privilege || authorization.requires_privilege {
        return Err("privileged actions are not executable on the current path".to_owned());
    }

    if confirmation.action_id != RECHECK_CONDITION_ACTION_ID {
        return Err(format!(
            "action is not supported by the current execution capability set: {}",
            confirmation.action_id
        ));
    }

    let eligibility = evaluate_execution_eligibility(confirmation);

    if !eligibility.eligible {
        return Err(eligibility.message);
    }

    Ok(AlertActionExecutionRequest {
        action_id: confirmation.action_id.clone(),
    })
}

/// Execute one explicitly allowlisted, non-privileged action.
///
/// The first execution capability is intentionally a bounded verification
/// operation. It does not invoke a shell or accept arbitrary system commands.
pub fn execute_alert_action(
    request: &AlertActionExecutionRequest,
) -> Result<AlertActionExecutionResult, String> {
    if request.action_id != RECHECK_CONDITION_ACTION_ID {
        return Err(format!(
            "action is not supported by the current execution capability set: {}",
            request.action_id
        ));
    }

    Ok(AlertActionExecutionResult {
        action_id: request.action_id.clone(),
        executed: true,
        message: "Recheck-condition execution completed without privileged system changes."
            .to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirmation(
        confirmed: bool,
        authorized: bool,
        executable: bool,
    ) -> AlertActionConfirmation {
        AlertActionConfirmation {
            action_id: RECHECK_CONDITION_ACTION_ID.to_owned(),
            title: "Recheck condition".to_owned(),
            confirmed,
            authorized,
            executable,
            requires_privilege: false,
            message: String::new(),
        }
    }

    fn authorization(action_id: &str, authorized: bool) -> AlertActionAuthorization {
        AlertActionAuthorization {
            action_id: action_id.to_owned(),
            authorized,
            requires_privilege: false,
            message: String::new(),
        }
    }

    #[test]
    fn confirmation_alone_is_not_execution_eligible() {
        let result = evaluate_execution_eligibility(&confirmation(true, false, false));
        assert!(!result.eligible);
        assert!(!result.authorized);
        assert!(!result.executable);
    }

    #[test]
    fn eligibility_requires_all_non_privileged_gates() {
        let result = evaluate_execution_eligibility(&confirmation(true, true, true));
        assert!(result.eligible);
    }

    #[test]
    fn privileged_confirmation_is_not_eligible() {
        let mut value = confirmation(true, true, true);
        value.requires_privilege = true;
        let result = evaluate_execution_eligibility(&value);
        assert!(!result.eligible);
    }

    #[test]
    fn execution_request_requires_authorization() {
        let confirmation = confirmation(true, true, true);

        let result = create_execution_request(
            &confirmation,
            &authorization(RECHECK_CONDITION_ACTION_ID, false),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "execution requires explicit action authorization"
        );
    }

    #[test]
    fn execution_request_requires_eligibility() {
        let confirmation = confirmation(true, true, false);

        let result = create_execution_request(
            &confirmation,
            &authorization(RECHECK_CONDITION_ACTION_ID, true),
        );

        assert!(result.is_err());
    }

    #[test]
    fn execution_request_rejects_mismatched_action_ids() {
        let confirmation = confirmation(true, true, true);

        let result =
            create_execution_request(&confirmation, &authorization("delete-everything", true));

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "authorization does not match the confirmed action"
        );
    }

    #[test]
    fn execution_request_accepts_only_allowlisted_action() {
        let confirmation = confirmation(true, true, true);
        let mut authorization = authorization(RECHECK_CONDITION_ACTION_ID, true);
        authorization.action_id = "delete-everything".to_owned();

        let result = create_execution_request(&confirmation, &authorization);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "authorization does not match the confirmed action"
        );
    }

    #[test]
    fn execution_request_rejects_unknown_confirmed_action() {
        let mut confirmation = confirmation(true, true, true);
        confirmation.action_id = "delete-everything".to_owned();

        let result =
            create_execution_request(&confirmation, &authorization("delete-everything", true));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("action is not supported by the current execution capability set")
        );
    }

    #[test]
    fn allowed_action_executes_without_privilege() {
        let request = AlertActionExecutionRequest {
            action_id: RECHECK_CONDITION_ACTION_ID.to_owned(),
        };

        let result = execute_alert_action(&request).expect("allowlisted action should execute");

        assert!(result.executed);
        assert_eq!(result.action_id, RECHECK_CONDITION_ACTION_ID);
    }

    #[test]
    fn unknown_action_cannot_execute_directly() {
        let request = AlertActionExecutionRequest {
            action_id: "delete-everything".to_owned(),
        };

        let result = execute_alert_action(&request);

        assert!(result.is_err());
    }
}
