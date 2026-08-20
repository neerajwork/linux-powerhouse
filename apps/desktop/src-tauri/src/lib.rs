mod action_audit;
mod action_remediation;
mod action_verification;
mod alert_history_store;

use action_audit::{ActionAudit, ActionAuditEntry};
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
    AlertActionPreview, AlertDecision, AlertEvent, AlertEventHistory, AlertGuidance, AlertState,
    HealthLevel, HealthSnapshot, PerformanceAnomalyReport, SignalKind, alert_decision,
    create_alert_event, explain_performance, guide_alert, preview_alert_actions,
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
        "refresh_health" => execute_unified_system_intelligence(&context(), "/".to_owned())
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
        "storage_diagnostic" => execute_storage_analysis(
            &user_confirmed_context(),
            "/".to_owned(),
            ScanLimits::default(),
        )
        .map(|_| SafeActionResult {
            action,
            status: "completed".to_owned(),
            message: "Storage diagnostic completed without changing system state.".to_owned(),
            reversible: true,
            privilege: "None".to_owned(),
            verification_status: String::new(),
            verification_message: String::new(),
        })
        .map_err(|error| error.to_string()),
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
            state.audit.record(
                &action_name,
                "verified",
                true,
                &result.status,
                &result.message,
                result.reversible,
                &result.privilege,
                &verification.status,
                &verification.message,
            )?;
            Ok(result)
        }
        Err(error) => {
            let verification = verify_safe_action(&action_name, false);
            let _ = state.audit.record(
                &action_name,
                "failed",
                true,
                "failed",
                &error,
                false,
                "Unknown",
                &verification.status,
                &verification.message,
            );
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
            clear_alert_event_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running Linux Powerhouse");
}
