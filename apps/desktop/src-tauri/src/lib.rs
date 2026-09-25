mod action_audit;
mod action_remediation;
mod action_verification;
mod alert_history_store;

use action_audit::{ActionAudit, ActionAuditEntry, ActionAuditRecord};
use action_remediation::{RemediationSuggestion, suggest_remediation};
use action_verification::verify_safe_action;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

use execution_engine::{
    execute_health_status, execute_monitoring_snapshot, execute_network_analysis,
    execute_network_status, execute_process_analysis, execute_process_status,
    execute_service_analysis, execute_storage_analysis, execute_storage_status,
    execute_system_status, execute_unified_system_intelligence,
};
use health_status::{
    AlertActionConfirmation, AlertActionExecutionRequest, AlertActionExecutionResult,
    AlertActionPreview, AlertDecision, AlertEvent, AlertEventHistory, AlertGuidance, AlertState,
    HealthLevel, HealthSnapshot, PerformanceAnomalyReport, SignalKind, alert_decision,
    confirm_alert_action, create_alert_event, derive_action_outcome, explain_performance,
    guide_alert, preview_alert_actions,
};
use monitoring::{Monitor, MonitorSnapshot, PerformanceHistoryComparison};
use network_intelligence::NetworkAnalysis;
use policy_engine::PolicyContext;
use process_intelligence::ProcessAnalysis;
use service_intelligence::ServiceAnalysis;
use storage_intelligence::{ScanLimits, StorageAnalysis};
use unified_system_intelligence::SystemIntelligenceSnapshot;

struct AppState {
    monitor: Mutex<Monitor>,
    audit: ActionAudit,
    alert_history: Mutex<AlertEventHistory>,
    alert_history_path: Mutex<Option<PathBuf>>,
}

#[derive(Debug, serde::Serialize)]
struct PerformanceDrilldown {
    performance: PerformanceAnomalyReport,
    processes: ProcessAnalysis,
}

#[derive(serde::Serialize)]
struct SafeActionResult {
    action: String,
    status: String,
    message: String,
    reversible: bool,
    privilege: String,
    verification_status: String,
    verification_message: String,
}

fn context() -> PolicyContext {
    PolicyContext {
        ai_requested: false,
        user_confirmed: false,
    }
}

fn user_confirmed_context() -> PolicyContext {
    PolicyContext {
        ai_requested: false,
        user_confirmed: true,
    }
}

fn persist_alert_history(state: &AppState) -> Result<(), String> {
    let path = state
        .alert_history_path
        .lock()
        .map_err(|_| "alert history path unavailable".to_owned())?
        .clone()
        .ok_or_else(|| "alert history persistence is not initialized".to_owned())?;
    let history = state
        .alert_history
        .lock()
        .map_err(|_| "alert history unavailable".to_owned())?
        .clone();
    alert_history_store::save(&path, &history)
}

#[tauri::command]
fn system_status() -> Result<system_status::SystemStatus, String> {
    execute_system_status(&context())
        .map(|result| result.status)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn system_intelligence(storage_root: String) -> Result<SystemIntelligenceSnapshot, String> {
    execute_unified_system_intelligence(&context(), storage_root).map_err(|error| error.to_string())
}

#[tauri::command]
fn storage_status() -> Result<Vec<storage_status::FilesystemStatus>, String> {
    execute_storage_status(&context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn storage_analyze(path: String) -> Result<StorageAnalysis, String> {
    execute_storage_analysis(&user_confirmed_context(), path, ScanLimits::default())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn process_status() -> Result<Vec<process_status::ProcessInfo>, String> {
    execute_process_status(&context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn process_analyze() -> Result<ProcessAnalysis, String> {
    execute_process_analysis(&user_confirmed_context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn network_status() -> Result<Vec<network_status::NetworkInterface>, String> {
    execute_network_status(&context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn network_analyze() -> Result<NetworkAnalysis, String> {
    execute_network_analysis(&user_confirmed_context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn service_analyze() -> Result<ServiceAnalysis, String> {
    execute_service_analysis(&user_confirmed_context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn safe_system_action(
    action: String,
    confirmed: bool,
    state: tauri::State<'_, AppState>,
) -> Result<SafeActionResult, String> {
    if !confirmed {
        return Err("explicit user confirmation is required".to_owned());
    }

    let action_name = action.clone();
    let outcome = match action.as_str() {
        "refresh_health" => execute_unified_system_intelligence(&context(), "/")
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "System health was refreshed.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
                verification_status: String::new(),
                verification_message: String::new(),
            })
            .map_err(|error| error.to_string()),
        "storage_diagnostic" => {
            execute_storage_analysis(&user_confirmed_context(), "/", ScanLimits::default())
                .map(|_| SafeActionResult {
                    action,
                    status: "completed".to_owned(),
                    message: "Storage diagnostic completed without changing system state."
                        .to_owned(),
                    reversible: true,
                    privilege: "None".to_owned(),
                    verification_status: String::new(),
                    verification_message: String::new(),
                })
                .map_err(|error| error.to_string())
        }
        "process_diagnostic" => execute_process_analysis(&user_confirmed_context())
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "Process diagnostic completed without changing system state.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
                verification_status: String::new(),
                verification_message: String::new(),
            })
            .map_err(|error| error.to_string()),
        "network_diagnostic" => execute_network_analysis(&user_confirmed_context())
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "Network diagnostic completed without changing system state.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
                verification_status: String::new(),
                verification_message: String::new(),
            })
            .map_err(|error| error.to_string()),
        "service_diagnostic" => execute_service_analysis(&user_confirmed_context())
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "Service diagnostic completed without changing system state.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
                verification_status: String::new(),
                verification_message: String::new(),
            })
            .map_err(|error| error.to_string()),
        _ => Err("action is not in the safe system-action allowlist".to_owned()),
    };

    match outcome {
        Ok(mut result) => {
            let verification = verify_safe_action(&action_name, true);
            result.verification_status = verification.status.clone();
            result.verification_message = verification.message.clone();

            let execution = AlertActionExecutionResult {
                action_id: action_name.clone(),
                executed: true,
                message: result.message.clone(),
            };

            let execution_request = AlertActionExecutionRequest {
                action_id: action_name.clone(),
            };

            let outcome = derive_action_outcome(
                &execution_request,
                execution,
                health_status::AlertActionVerificationResult {
                    action_id: action_name.clone(),
                    status: if verification.status == "verified" {
                        health_status::AlertActionVerificationStatus::Passed
                    } else {
                        health_status::AlertActionVerificationStatus::Failed
                    },
                    message: verification.message.clone(),
                },
            );

            state.audit.record(&ActionAuditRecord {
                action: &action_name,
                stage: "verified",
                confirmed: true,
                status: &result.status,
                message: &result.message,
                reversible: result.reversible,
                privilege: &result.privilege,
                verification_status: &verification.status,
                verification_message: &verification.message,
                outcome: &outcome,
            })?;
            Ok(result)
        }
        Err(error) => {
            let verification = verify_safe_action(&action_name, false);

            let execution = AlertActionExecutionResult {
                action_id: action_name.clone(),
                executed: false,
                message: error.clone(),
            };

            let execution_request = AlertActionExecutionRequest {
                action_id: action_name.clone(),
            };

            let verification_result = health_status::AlertActionVerificationResult {
                action_id: action_name.clone(),
                status: health_status::AlertActionVerificationStatus::Failed,
                message: verification.message.clone(),
            };

            let outcome = derive_action_outcome(&execution_request, execution, verification_result);

            state.audit.record(&ActionAuditRecord {
                action: &action_name,
                stage: "failed",
                confirmed: true,
                status: "failed",
                message: &error,
                reversible: false,
                privilege: "Unknown",
                verification_status: &verification.status,
                verification_message: &verification.message,
                outcome: &outcome,
            })?;

            Err(error)
        }
    }
}
#[tauri::command]
fn action_audit_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ActionAuditEntry>, String> {
    state.audit.history()
}

#[tauri::command]
fn action_remediation_suggestions(
    action: String,
    status: String,
    verification_status: String,
) -> Vec<RemediationSuggestion> {
    suggest_remediation(&action, &status, &verification_status)
}

#[tauri::command]
fn monitor_snapshot(state: tauri::State<'_, AppState>) -> Result<MonitorSnapshot, String> {
    let mut monitor = state
        .monitor
        .lock()
        .map_err(|_| "monitor state unavailable".to_owned())?;
    execute_monitoring_snapshot(&context(), &mut monitor).map_err(|error| error.to_string())
}

#[tauri::command]
fn monitor_history(state: tauri::State<'_, AppState>) -> Result<Vec<MonitorSnapshot>, String> {
    let monitor = state
        .monitor
        .lock()
        .map_err(|_| "monitor state unavailable".to_owned())?;
    Ok(monitor.history())
}

#[tauri::command]
fn performance_history_comparison(
    state: tauri::State<'_, AppState>,
) -> Result<PerformanceHistoryComparison, String> {
    let monitor = state
        .monitor
        .lock()
        .map_err(|_| "monitor state unavailable".to_owned())?;
    monitor
        .performance_history_comparison()
        .ok_or_else(|| "insufficient performance history for comparison".to_owned())
}

#[tauri::command]
fn performance_anomaly_explanations(
    state: tauri::State<'_, AppState>,
) -> Result<PerformanceAnomalyReport, String> {
    let monitor = state
        .monitor
        .lock()
        .map_err(|_| "monitor state unavailable".to_owned())?;
    let snapshot = monitor
        .history()
        .last()
        .cloned()
        .ok_or_else(|| "no monitoring snapshot is available".to_owned())?;
    Ok(explain_performance(&snapshot))
}

#[tauri::command]
fn process_performance_drilldown(
    state: tauri::State<'_, AppState>,
) -> Result<PerformanceDrilldown, String> {
    let monitor = state
        .monitor
        .lock()
        .map_err(|_| "monitor state unavailable".to_owned())?;
    let snapshot = monitor
        .history()
        .last()
        .cloned()
        .ok_or_else(|| "no monitoring snapshot is available".to_owned())?;
    let processes = process_intelligence::analyze().map_err(|error| error.to_string())?;
    Ok(PerformanceDrilldown {
        performance: explain_performance(&snapshot),
        processes,
    })
}

#[tauri::command]
fn health_status(state: tauri::State<'_, AppState>) -> Result<HealthSnapshot, String> {
    let mut monitor = state
        .monitor
        .lock()
        .map_err(|_| "monitor state unavailable".to_owned())?;
    let snapshot = monitor.snapshot().map_err(|error| error.to_string())?;
    let storage =
        execution_engine::execute_storage_status(&context()).map_err(|error| error.to_string())?;
    let max_storage_usage = storage.iter().map(|item| item.usage_percent).max();
    execute_health_status(&context(), Some(&snapshot), max_storage_usage)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn alert_decision_for_signal(
    kind: SignalKind,
    level: HealthLevel,
    state: AlertState,
    value: f64,
    now_ms: u128,
    app_state: tauri::State<'_, AppState>,
) -> Result<Option<AlertDecision>, String> {
    let decision = alert_decision(kind, level, state, now_ms);

    if let Some(decision) = decision {
        if let Some(event) = create_alert_event(now_ms, kind, level, value, state, decision) {
            app_state
                .alert_history
                .lock()
                .map_err(|_| "alert history unavailable".to_owned())?
                .record(event);
            persist_alert_history(&app_state)?;
        }

        Ok(Some(decision))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn alert_event_history(state: tauri::State<'_, AppState>) -> Result<Vec<AlertEvent>, String> {
    Ok(state
        .alert_history
        .lock()
        .map_err(|_| "alert history unavailable".to_owned())?
        .events()
        .to_vec())
}

#[tauri::command]
fn alert_guidance(event: AlertEvent) -> AlertGuidance {
    guide_alert(&event)
}

#[tauri::command]
fn alert_action_preview(event: AlertEvent) -> Vec<AlertActionPreview> {
    preview_alert_actions(&event)
}

#[tauri::command]
fn alert_action_confirm(
    event: AlertEvent,
    action_id: String,
) -> Result<AlertActionConfirmation, String> {
    confirm_alert_action(&event, &action_id)
}

#[tauri::command]
fn clear_alert_event_history(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state
        .alert_history
        .lock()
        .map_err(|_| "alert history unavailable".to_owned())?
        .clear();
    persist_alert_history(&state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            monitor: Mutex::new(Monitor::new()),
            audit: ActionAudit,
            alert_history: Mutex::new(AlertEventHistory::default()),
            alert_history_path: Mutex::new(None),
        })
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            std::fs::create_dir_all(&app_data_dir)?;
            let history_path = alert_history_store::path(&app_data_dir);
            let history = alert_history_store::load(&history_path);
            let state = app.state::<AppState>();
            *state
                .alert_history
                .lock()
                .map_err(|_| std::io::Error::other("alert history unavailable"))? = history;
            *state
                .alert_history_path
                .lock()
                .map_err(|_| std::io::Error::other("alert history path unavailable"))? =
                Some(history_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            system_status,
            system_intelligence,
            storage_status,
            storage_analyze,
            process_status,
            process_analyze,
            network_status,
            network_analyze,
            service_analyze,
            safe_system_action,
            action_audit_history,
            action_remediation_suggestions,
            monitor_snapshot,
            monitor_history,
            performance_history_comparison,
            performance_anomaly_explanations,
            process_performance_drilldown,
            health_status,
            alert_decision_for_signal,
            alert_event_history,
            alert_guidance,
            alert_action_preview,
            alert_action_confirm,
            clear_alert_event_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running Linux Powerhouse");
}

#[cfg(test)]
mod remediation_command_tests {
    use super::action_remediation_suggestions;

    #[test]
    fn remediation_command_preserves_verified_action_suggestion() {
        let suggestions = action_remediation_suggestions(
            "storage_diagnostic".to_owned(),
            "completed".to_owned(),
            "verified".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].action, "storage_diagnostic");
        assert_eq!(suggestions[0].suggested_action, "refresh_health");
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn remediation_command_preserves_successful_verified_action_suggestion() {
        let suggestions = action_remediation_suggestions(
            "storage_diagnostic".to_owned(),
            "success".to_owned(),
            "verified".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].action, "storage_diagnostic");
        assert_eq!(suggestions[0].suggested_action, "refresh_health");
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn remediation_command_preserves_completed_verified_success_reason() {
        let suggestions = action_remediation_suggestions(
            "storage_diagnostic".to_owned(),
            "completed".to_owned(),
            "verified".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].reason,
            "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state."
        );
    }

    #[test]
    fn remediation_command_preserves_successful_verified_success_reason() {
        let suggestions = action_remediation_suggestions(
            "storage_diagnostic".to_owned(),
            "success".to_owned(),
            "verified".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].reason,
            "The read-only action completed successfully; a fresh health refresh can confirm the latest overall state."
        );
    }

    #[test]
    fn remediation_command_preserves_failed_action_reason() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "failed".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].reason,
            "The action did not complete successfully, so a safe follow-up diagnostic is recommended."
        );
    }

    #[test]
    fn remediation_command_preserves_completed_verified_confirmation_requirement() {
        let suggestions = action_remediation_suggestions(
            "storage_diagnostic".to_owned(),
            "completed".to_owned(),
            "verified".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn remediation_command_preserves_successful_verified_confirmation_requirement() {
        let suggestions = action_remediation_suggestions(
            "storage_diagnostic".to_owned(),
            "success".to_owned(),
            "verified".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn remediation_command_preserves_failed_confirmation_requirement() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "failed".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn remediation_command_preserves_completed_verified_diagnostic_actions() {
        for action in [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let suggestions = action_remediation_suggestions(
                action.to_owned(),
                "completed".to_owned(),
                "verified".to_owned(),
            );

            assert_eq!(suggestions.len(), 1);
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, "refresh_health");
            assert!(suggestions[0].requires_confirmation);
        }
    }

    #[test]
    fn remediation_command_preserves_successful_verified_diagnostic_actions() {
        for action in [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let suggestions = action_remediation_suggestions(
                action.to_owned(),
                "success".to_owned(),
                "verified".to_owned(),
            );

            assert_eq!(suggestions.len(), 1);
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, "refresh_health");
            assert!(suggestions[0].requires_confirmation);
        }
    }

    #[test]
    fn remediation_command_preserves_failed_action_suggestion() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "failed".to_owned(),
        );

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].action, "refresh_health");
        assert_eq!(suggestions[0].suggested_action, "storage_diagnostic");
        assert!(suggestions[0].requires_confirmation);
    }

    #[test]
    fn remediation_command_preserves_failed_diagnostic_action_suggestions() {
        for action in [
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ] {
            let suggestions = action_remediation_suggestions(
                action.to_owned(),
                "failed".to_owned(),
                "failed".to_owned(),
            );

            assert_eq!(suggestions.len(), 1, "expected one suggestion for {action}");
            assert_eq!(suggestions[0].action, action);
            assert_eq!(suggestions[0].suggested_action, "refresh_health");
            assert!(suggestions[0].requires_confirmation);
        }
    }

    #[test]
    fn remediation_command_rejects_failed_action_with_verified_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "verified".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_failed_action_with_whitespace_padded_failed_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            " failed ".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_successful_action_with_failed_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "completed".to_owned(),
            "failed".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_success_action_with_failed_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "success".to_owned(),
            "failed".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_preserves_no_suggestion_for_incomplete_action() {
        let suggestions = action_remediation_suggestions(
            "storage_diagnostic".to_owned(),
            "pending".to_owned(),
            "legacy".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_preserves_no_suggestion_for_unknown_status() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "unknown".to_owned(),
            "verified".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_preserves_no_suggestion_for_unknown_verification_status() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "completed".to_owned(),
            "unknown".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_failed_action_with_unknown_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "unknown".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_failed_action_with_legacy_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "legacy".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_failed_action_with_pending_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "pending".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_failed_action_with_empty_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_failed_action_with_whitespace_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "failed".to_owned(),
            "   ".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_completed_action_with_legacy_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "completed".to_owned(),
            "legacy".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_completed_action_with_pending_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "completed".to_owned(),
            "pending".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_completed_action_with_empty_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "completed".to_owned(),
            "".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_completed_action_with_whitespace_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "completed".to_owned(),
            "   ".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_completed_action_with_whitespace_padded_verified_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "completed".to_owned(),
            " verified ".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_success_action_with_unknown_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "success".to_owned(),
            "unknown".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_success_action_with_legacy_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "success".to_owned(),
            "legacy".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_success_action_with_pending_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "success".to_owned(),
            "pending".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_success_action_with_empty_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "success".to_owned(),
            "".to_owned(),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn remediation_command_rejects_success_action_with_whitespace_verification() {
        let suggestions = action_remediation_suggestions(
            "refresh_health".to_owned(),
            "success".to_owned(),
            "   ".to_owned(),
        );

        assert!(suggestions.is_empty());
    }
}
