mod action_audit;

use action_audit::{ActionAudit, ActionAuditEntry};
use std::sync::Mutex;

use execution_engine::{
    execute_health_status, execute_monitoring_snapshot, execute_network_analysis,
    execute_network_status, execute_process_analysis, execute_process_status,
    execute_service_analysis, execute_storage_analysis, execute_storage_status,
    execute_system_status, execute_unified_system_intelligence,
};
use health_status::HealthSnapshot;
use monitoring::{Monitor, MonitorSnapshot};
use network_intelligence::NetworkAnalysis;
use policy_engine::PolicyContext;
use process_intelligence::ProcessAnalysis;
use service_intelligence::ServiceAnalysis;
use storage_intelligence::{ScanLimits, StorageAnalysis};
use unified_system_intelligence::SystemIntelligenceSnapshot;

struct AppState {
    monitor: Mutex<Monitor>,
    audit: ActionAudit,
}

#[derive(serde::Serialize)]
struct SafeActionResult {
    action: String,
    status: String,
    message: String,
    reversible: bool,
    privilege: String,
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

#[tauri::command]
fn system_status() -> Result<system_status::SystemStatus, String> {
    execute_system_status(&context())
        .map(|result| result.status)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn system_intelligence(storage_root: String) -> Result<SystemIntelligenceSnapshot, String> {
    execute_unified_system_intelligence(&context(), storage_root)
        .map_err(|error| error.to_string())
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
    state: tauri::State<'_, AppState>,
) -> Result<SafeActionResult, String> {
    let action_name = action.clone();
    let outcome = match action.as_str() {
        "refresh_health" => execute_unified_system_intelligence(&context(), "/".to_owned())
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "System health was refreshed.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
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
        })
        .map_err(|error| error.to_string()),
        "process_diagnostic" => execute_process_analysis(&user_confirmed_context())
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "Process diagnostic completed without changing system state.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
            })
            .map_err(|error| error.to_string()),
        "network_diagnostic" => execute_network_analysis(&user_confirmed_context())
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "Network diagnostic completed without changing system state.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
            })
            .map_err(|error| error.to_string()),
        "service_diagnostic" => execute_service_analysis(&user_confirmed_context())
            .map(|_| SafeActionResult {
                action,
                status: "completed".to_owned(),
                message: "Service diagnostic completed without changing system state.".to_owned(),
                reversible: true,
                privilege: "None".to_owned(),
            })
            .map_err(|error| error.to_string()),
        _ => Err("action is not in the safe system-action allowlist".to_owned()),
    };

    match outcome {
        Ok(result) => {
            state.audit.record(
                &action_name,
                "executed",
                true,
                &result.status,
                &result.message,
                result.reversible,
                &result.privilege,
            )?;
            Ok(result)
        }
        Err(error) => {
            let _ = state.audit.record(
                &action_name,
                "failed",
                true,
                "failed",
                &error,
                false,
                "Unknown",
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            monitor: Mutex::new(Monitor::new()),
            audit: ActionAudit,
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
            monitor_snapshot,
            monitor_history,
            health_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Linux Powerhouse");
}
