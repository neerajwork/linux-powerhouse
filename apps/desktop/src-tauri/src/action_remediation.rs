#[derive(Clone, Debug, serde::Serialize)]
pub struct RemediationSuggestion {
    pub action: String,
    pub reason: String,
    pub suggested_action: String,
    pub requires_confirmation: bool,
}

pub fn suggest_remediation(
    action: &str,
    status: &str,
    verification_status: &str,
) -> Vec<RemediationSuggestion> {
    if !matches!(status, "success" | "completed" | "failed") {
        return Vec::new();
    }

    if status != "failed" && verification_status == "failed" {
        return Vec::new();
    }

    if status == "failed" && !matches!(verification_status, "failed") {
        return Vec::new();
    }

    if status == "failed" || verification_status == "failed" {
        let suggested_action = match action {
            "refresh_health" => "storage_diagnostic",
            "storage_diagnostic" => "refresh_health",
            "process_diagnostic" => "refresh_health",
            "network_diagnostic" => "refresh_health",
            "service_diagnostic" => "refresh_health",
            _ => "refresh_health",
        };
        return vec![RemediationSuggestion {
            action: action.to_owned(),
            reason: "The action did not complete successfully, so a safe follow-up diagnostic is recommended.".to_owned(),
            suggested_action: suggested_action.to_owned(),
            requires_confirmation: true,
        }];
    }

    if verification_status == "verified" {
        return vec![RemediationSuggestion {
            action: action.to_owned(),
            reason: "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state.".to_owned(),
            suggested_action: "refresh_health".to_owned(),
            requires_confirmation: true,
        }];
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::suggest_remediation;

    #[test]
    fn failed_refresh_health_suggests_storage_diagnostic() {
        let suggestions = suggest_remediation("refresh_health", "failed", "failed");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].action, "refresh_health");
        assert_eq!(suggestions[0].suggested_action, "storage_diagnostic");
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn failed_diagnostic_actions_suggest_health_refresh() {
        for action in [
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let suggestions = suggest_remediation(action, "failed", "failed");

            assert_eq!(suggestions.len(), 1, "expected one suggestion for {action}");
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, "refresh_health");
            assert!(suggestions[0].requires_confirmation);
        }
    }

    #[test]
    fn failed_unknown_action_suggests_health_refresh() {
        let suggestions = suggest_remediation("unknown_action", "failed", "failed");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].action, "unknown_action");
        assert_eq!(suggestions[0].suggested_action, "refresh_health");
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn verified_action_suggests_health_refresh() {
        let suggestions = suggest_remediation("storage_diagnostic", "completed", "verified");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].action, "storage_diagnostic");
        assert_eq!(suggestions[0].suggested_action, "refresh_health");
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn successful_action_with_verified_verification_suggests_health_refresh() {
        let suggestions = suggest_remediation("storage_diagnostic", "success", "verified");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].action, "storage_diagnostic");
        assert_eq!(suggestions[0].suggested_action, "refresh_health");
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn completed_diagnostic_actions_with_verified_verification_suggest_health_refresh() {
        for action in [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let suggestions = suggest_remediation(action, "completed", "verified");

            assert_eq!(suggestions.len(), 1);
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, "refresh_health");
            assert!(suggestions[0].requires_confirmation);
        }
    }

    #[test]
    fn successful_diagnostic_actions_with_verified_verification_suggest_health_refresh() {
        for action in [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let suggestions = suggest_remediation(action, "success", "verified");

            assert_eq!(suggestions.len(), 1);
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, "refresh_health");
            assert!(suggestions[0].requires_confirmation);
        }
    }

    #[test]
    fn completed_verified_action_has_success_reason() {
        let suggestions = suggest_remediation("storage_diagnostic", "completed", "verified");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].reason,
            "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state."
        );
    }

    #[test]
    fn successful_verified_action_has_success_reason() {
        let suggestions = suggest_remediation("storage_diagnostic", "success", "verified");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].reason,
            "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state."
        );
    }

    #[test]
    fn failed_action_has_failure_reason() {
        let suggestions = suggest_remediation("refresh_health", "failed", "failed");

        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].reason,
            "The action did not complete successfully, so a safe follow-up diagnostic is recommended."
        );
    }

    #[test]
    fn failed_action_with_verified_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "failed", "verified");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn failed_action_with_unknown_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "failed", "unknown");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn failed_action_with_legacy_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "failed", "legacy");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn failed_action_with_pending_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "failed", "pending");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn failed_action_with_empty_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "failed", "");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn failed_action_with_whitespace_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "failed", "   ");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn failed_action_with_whitespace_padded_failed_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "failed", " failed ");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn successful_action_with_failed_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "completed", "failed");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn incomplete_action_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "completed", "pending");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn unknown_status_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "unknown", "verified");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn unknown_verification_status_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "completed", "unknown");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn completed_action_with_legacy_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "completed", "legacy");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn completed_action_with_pending_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "completed", "pending");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn completed_action_with_empty_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "completed", "");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn completed_action_with_whitespace_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "completed", "   ");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn completed_action_with_whitespace_padded_verified_verification_has_no_remediation_suggestion()
    {
        let suggestions = suggest_remediation("refresh_health", "completed", " verified ");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn successful_action_with_unknown_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "success", "unknown");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn successful_action_with_whitespace_padded_verified_verification_has_no_remediation_suggestion()
     {
        let suggestions = suggest_remediation("refresh_health", "success", " verified ");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn successful_action_with_legacy_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "success", "legacy");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn successful_action_with_pending_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "success", "pending");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn successful_action_with_empty_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "success", "");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn successful_action_with_whitespace_verification_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("refresh_health", "success", "   ");

        assert!(suggestions.is_empty());
    }
}
