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
