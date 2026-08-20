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