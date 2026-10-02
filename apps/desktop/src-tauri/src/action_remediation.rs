#[derive(Clone, Debug, PartialEq, serde::Serialize)]
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

    if verification_status == "verified"
        && !matches!(
            action,
            "refresh_health"
                | "storage_diagnostic"
                | "process_diagnostic"
                | "network_diagnostic"
                | "service_diagnostic"
        )
    {
        return Vec::new();
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
    fn failed_actions_suggest_expected_follow_up_actions() {
        let expected = [
            ("refresh_health", "storage_diagnostic"),
            ("storage_diagnostic", "refresh_health"),
            ("process_diagnostic", "refresh_health"),
            ("network_diagnostic", "refresh_health"),
            ("service_diagnostic", "refresh_health"),
            ("unknown_action", "refresh_health"),
            ("unsupported_action", "refresh_health"),
            ("future_action", "refresh_health"),
        ];

        for (action, suggested_action) in expected {
            let suggestions = suggest_remediation(action, "failed", "failed");

            assert_eq!(suggestions.len(), 1, "expected one suggestion for {action}");
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, suggested_action);
            assert!(suggestions[0].requires_confirmation);
            assert_eq!(
                suggestions[0].reason,
                "The action did not complete successfully, so a safe follow-up diagnostic is recommended."
            );
        }
    }
    #[test]
    fn failed_unknown_actions_suggest_health_refresh() {
        for action in ["unknown_action", "unsupported_action", "future_action"] {
            let suggestions = suggest_remediation(action, "failed", "failed");

            assert_eq!(suggestions.len(), 1, "expected one suggestion for {action}");
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, "refresh_health");
            assert!(suggestions[0].requires_confirmation);
            assert_eq!(
                suggestions[0].reason,
                "The action did not complete successfully, so a safe follow-up diagnostic is recommended."
            );
        }
    }

    #[test]
    fn failed_actions_only_remediate_with_failed_verification() {
        for action in [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let failed = suggest_remediation(action, "failed", "failed");
            assert_eq!(
                failed.len(),
                1,
                "expected remediation for failed/{action}/failed"
            );

            for verification_status in ["verified", "pending", "legacy", "unknown", ""] {
                let suggestions = suggest_remediation(action, "failed", verification_status);

                assert!(
                    suggestions.is_empty(),
                    "expected no remediation for failed/{action}/{verification_status:?}"
                );
            }
        }
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
    fn successful_unknown_action_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("unknown_action", "success", "verified");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn completed_unknown_action_has_no_remediation_suggestion() {
        let suggestions = suggest_remediation("unknown_action", "completed", "verified");

        assert!(suggestions.is_empty());
    }

    #[test]
    fn unknown_actions_have_no_success_remediation_contract() {
        for status in ["success", "completed"] {
            for action in ["unknown_action", "unsupported_action", "future_action"] {
                let suggestions = suggest_remediation(action, status, "verified");

                assert!(
                    suggestions.is_empty(),
                    "expected no remediation for {status}/verified/{action}"
                );
            }
        }
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
    fn successful_actions_have_consistent_success_reason() {
        let expected_reason = "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state.";

        for status in ["completed", "success"] {
            for action in [
                "refresh_health",
                "storage_diagnostic",
                "process_diagnostic",
                "network_diagnostic",
                "service_diagnostic",
            ] {
                let suggestions = suggest_remediation(action, status, "verified");

                assert_eq!(
                    suggestions.len(),
                    1,
                    "expected one suggestion for {status}/{action}"
                );
                assert_eq!(suggestions[0].reason, expected_reason);
            }
        }
    }

    #[test]
    fn successful_actions_have_consistent_remediation_contract() {
        let expected_reason = "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state.";

        let expected_actions = [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ];

        for status in ["completed", "success"] {
            for action in expected_actions {
                let suggestions = suggest_remediation(action, status, "verified");

                assert_eq!(
                    suggestions.len(),
                    1,
                    "expected one suggestion for {status}/{action}"
                );
                assert_eq!(suggestions[0].action, action);
                assert_eq!(suggestions[0].suggested_action, "refresh_health");
                assert_eq!(suggestions[0].reason, expected_reason);
                assert!(suggestions[0].requires_confirmation);
            }
        }
    }

    #[test]
    fn remediation_suggestion_serializes_expected_fields() {
        let suggestions = suggest_remediation("storage_diagnostic", "success", "verified");

        assert_eq!(suggestions.len(), 1);

        let serialized = serde_json::to_value(&suggestions[0]).expect("expected valid JSON");

        assert_eq!(
            serialized,
            serde_json::json!({
                "action": "storage_diagnostic",
                "reason": "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state.",
                "suggested_action": "refresh_health",
                "requires_confirmation": true,
            })
        );
    }

    #[test]
    fn failed_remediation_suggestion_serializes_expected_fields() {
        let suggestions = suggest_remediation("storage_diagnostic", "failed", "failed");

        assert_eq!(suggestions.len(), 1);

        let serialized = serde_json::to_value(&suggestions[0]).expect("expected valid JSON");

        assert_eq!(
            serialized,
            serde_json::json!({
                "action": "storage_diagnostic",
                "reason": "The action did not complete successfully, so a safe follow-up diagnostic is recommended.",
                "suggested_action": "refresh_health",
                "requires_confirmation": true,
            })
        );
    }

    #[test]
    fn unknown_failed_remediation_serializes_expected_values() {
        let suggestions = suggest_remediation("unknown_action", "failed", "failed");

        assert_eq!(suggestions.len(), 1);

        let serialized = serde_json::to_value(&suggestions[0]).expect("expected valid JSON");

        assert_eq!(
            serialized,
            serde_json::json!({
                "action": "unknown_action",
                "reason": "The action did not complete successfully, so a safe follow-up diagnostic is recommended.",
                "suggested_action": "refresh_health",
                "requires_confirmation": true,
            })
        );
    }

    #[test]
    fn remediation_suggestions_serialize_consistently_for_supported_actions() {
        let expected_actions = [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ];

        for action in expected_actions {
            for (status, verification_status) in [
                ("success", "verified"),
                ("completed", "verified"),
                ("failed", "failed"),
            ] {
                let suggestions = suggest_remediation(action, status, verification_status);

                assert_eq!(
                    suggestions.len(),
                    1,
                    "expected one suggestion for {status}/{verification_status}/{action}"
                );

                let serialized =
                    serde_json::to_value(&suggestions[0]).expect("expected valid JSON");

                let object = serialized
                    .as_object()
                    .expect("expected serialized suggestion object");

                assert_eq!(
                    object.len(),
                    4,
                    "unexpected serialized field count for {status}/{verification_status}/{action}"
                );

                for field in [
                    "action",
                    "reason",
                    "suggested_action",
                    "requires_confirmation",
                ] {
                    assert!(
                        object.contains_key(field),
                        "missing serialized field {field} for {status}/{verification_status}/{action}"
                    );
                }
            }
        }
    }

    #[test]
    fn remediation_suggestions_serialize_consistently_with_expected_values() {
        let expected = [
            ("refresh_health", "storage_diagnostic"),
            ("storage_diagnostic", "refresh_health"),
            ("process_diagnostic", "refresh_health"),
            ("network_diagnostic", "refresh_health"),
            ("service_diagnostic", "refresh_health"),
        ];

        for (action, failed_suggested_action) in expected {
            for (status, verification_status, reason, suggested_action) in [
                (
                    "success",
                    "verified",
                    "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state.",
                    "refresh_health",
                ),
                (
                    "completed",
                    "verified",
                    "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state.",
                    "refresh_health",
                ),
                (
                    "failed",
                    "failed",
                    "The action did not complete successfully, so a safe follow-up diagnostic is recommended.",
                    failed_suggested_action,
                ),
            ] {
                let suggestions = suggest_remediation(action, status, verification_status);

                assert_eq!(suggestions.len(), 1);

                let serialized =
                    serde_json::to_value(&suggestions[0]).expect("expected valid JSON");

                assert_eq!(
                    serialized["action"],
                    serde_json::Value::String(action.to_owned())
                );
                assert_eq!(
                    serialized["reason"],
                    serde_json::Value::String(reason.to_owned())
                );
                assert_eq!(
                    serialized["suggested_action"],
                    serde_json::Value::String(suggested_action.to_owned())
                );
                assert_eq!(serialized["requires_confirmation"], true);
            }
        }
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
    fn remediation_actions_have_consistent_suggested_action() {
        let expected_failed = [
            ("refresh_health", "storage_diagnostic"),
            ("storage_diagnostic", "refresh_health"),
            ("process_diagnostic", "refresh_health"),
            ("network_diagnostic", "refresh_health"),
            ("service_diagnostic", "refresh_health"),
        ];

        let expected_verified = [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ];

        for (action, expected_suggested_action) in expected_failed {
            let suggestions = suggest_remediation(action, "failed", "failed");

            assert_eq!(
                suggestions.len(),
                1,
                "expected one failed suggestion for {action}"
            );
            assert_eq!(
                suggestions[0].suggested_action, expected_suggested_action,
                "unexpected failed suggested action for {action}"
            );
        }

        for status in ["completed", "success"] {
            for action in expected_verified {
                let suggestions = suggest_remediation(action, status, "verified");

                assert_eq!(
                    suggestions.len(),
                    1,
                    "expected one verified suggestion for {status}/{action}"
                );
                assert_eq!(
                    suggestions[0].suggested_action, "refresh_health",
                    "unexpected verified suggested action for {status}/{action}"
                );
            }
        }
    }

    #[test]
    fn successful_and_completed_verified_actions_have_consistent_remediation() {
        for action in [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let completed = suggest_remediation(action, "completed", "verified");
            let successful = suggest_remediation(action, "success", "verified");

            assert_eq!(
                completed, successful,
                "expected identical remediation for completed/success/{action}"
            );
        }
    }

    #[test]
    fn completed_verified_action_requires_confirmation() {
        let suggestions = suggest_remediation("storage_diagnostic", "completed", "verified");

        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn successful_verified_action_requires_confirmation() {
        let suggestions = suggest_remediation("storage_diagnostic", "success", "verified");

        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn remediation_actions_consistently_require_confirmation() {
        for (status, verification_status) in [
            ("completed", "verified"),
            ("success", "verified"),
            ("failed", "failed"),
        ] {
            for action in [
                "refresh_health",
                "storage_diagnostic",
                "process_diagnostic",
                "network_diagnostic",
                "service_diagnostic",
            ] {
                let suggestions = suggest_remediation(action, status, verification_status);

                assert_eq!(
                    suggestions.len(),
                    1,
                    "expected one suggestion for {status}/{verification_status}/{action}"
                );
                assert!(
                    suggestions[0].requires_confirmation,
                    "expected confirmation for {status}/{verification_status}/{action}"
                );
            }
        }
    }

    #[test]
    fn failed_action_requires_confirmation() {
        let suggestions = suggest_remediation("refresh_health", "failed", "failed");

        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].requires_confirmation);
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
    fn failed_action_preserves_action_identity() {
        let expected = [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ];

        for action in expected {
            let suggestions = suggest_remediation(action, "failed", "failed");

            assert_eq!(suggestions.len(), 1, "expected one suggestion for {action}");
            assert_eq!(suggestions[0].action, action);
        }
    }

    #[test]
    fn failed_actions_have_consistent_failure_reason() {
        let expected_reason = "The action did not complete successfully, so a safe follow-up diagnostic is recommended.";

        for action in [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let suggestions = suggest_remediation(action, "failed", "failed");

            assert_eq!(suggestions.len(), 1, "expected one suggestion for {action}");
            assert_eq!(suggestions[0].reason, expected_reason);
        }
    }

    #[test]
    fn failed_actions_have_consistent_remediation_contract() {
        let expected_reason = "The action did not complete successfully, so a safe follow-up diagnostic is recommended.";

        let expected = [
            ("refresh_health", "storage_diagnostic"),
            ("storage_diagnostic", "refresh_health"),
            ("process_diagnostic", "refresh_health"),
            ("network_diagnostic", "refresh_health"),
            ("service_diagnostic", "refresh_health"),
            ("unknown_action", "refresh_health"),
            ("unsupported_action", "refresh_health"),
            ("future_action", "refresh_health"),
        ];

        for (action, expected_suggested_action) in expected {
            let suggestions = suggest_remediation(action, "failed", "failed");

            assert_eq!(suggestions.len(), 1, "expected one suggestion for {action}");
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, expected_suggested_action);
            assert_eq!(suggestions[0].reason, expected_reason);
            assert!(suggestions[0].requires_confirmation);
        }
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

    #[test]
    fn successful_and_completed_actions_only_remediate_with_verified_verification() {
        for status in ["success", "completed"] {
            for verification_status in [
                "failed",
                "pending",
                "legacy",
                "unknown",
                "",
                "   ",
                " verified ",
            ] {
                let suggestions =
                    suggest_remediation("refresh_health", status, verification_status);

                assert!(
                    suggestions.is_empty(),
                    "expected no remediation for {status}/{verification_status:?}"
                );
            }
        }
    }

    #[test]
    fn unsupported_status_never_produces_remediation() {
        for status in ["unknown", "pending", "running", "cancelled", ""] {
            for verification_status in ["verified", "failed", "pending", "legacy", "unknown", ""] {
                let suggestions =
                    suggest_remediation("refresh_health", status, verification_status);

                assert!(
                    suggestions.is_empty(),
                    "expected no remediation for {status:?}/{verification_status:?}"
                );
            }
        }
    }

    #[test]
    fn no_remediation_serializes_as_empty_array() {
        for (action, status, verification_status) in [
            ("refresh_health", "completed", "pending"),
            ("refresh_health", "failed", "verified"),
            ("refresh_health", "success", "unknown"),
            ("refresh_health", "unknown", "verified"),
            ("unknown_action", "success", "verified"),
        ] {
            let suggestions = suggest_remediation(action, status, verification_status);

            assert!(
                suggestions.is_empty(),
                "expected no remediation for {action:?}/{status:?}/{verification_status:?}"
            );

            let serialized =
                serde_json::to_value(&suggestions).expect("expected valid JSON serialization");

            assert_eq!(
                serialized,
                serde_json::json!([]),
                "expected empty JSON array for {action:?}/{status:?}/{verification_status:?}"
            );
        }
    }
}
