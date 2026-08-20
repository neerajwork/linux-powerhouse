use crate::AlertActionConfirmation;

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct AlertActionExecutionEligibility {
    pub action_id: String,
    pub eligible: bool,
    pub authorized: bool,
    pub executable: bool,
    pub requires_privilege: bool,
    pub message: String,
}

/// Establish the execution boundary after explicit confirmation.
///
/// Step 65 deliberately performs no execution. A confirmed preview remains
/// ineligible until a later milestone supplies a narrowly scoped execution
/// contract and authorization mechanism.
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
            "Execution is not eligible: explicit confirmation alone does not authorize or enable execution.".to_owned()
        },
    }
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
}
