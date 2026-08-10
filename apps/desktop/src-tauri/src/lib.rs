use execution_engine::{execute_network_status, execute_process_status, execute_storage_status, execute_system_status};
use policy_engine::PolicyContext;

fn context() -> PolicyContext {
    PolicyContext { ai_requested: false, user_confirmed: false }
}

#[tauri::command]
fn system_status() -> Result<system_status::SystemStatus, String> {
    execute_system_status(&context()).map(|result| result.status).map_err(|error| error.to_string())
}

#[tauri::command]
fn storage_status() -> Result<Vec<storage_status::FilesystemStatus>, String> {
    execute_storage_status(&context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn process_status() -> Result<Vec<process_status::ProcessInfo>, String> {
    execute_process_status(&context()).map_err(|error| error.to_string())
}

#[tauri::command]
fn network_status() -> Result<Vec<network_status::NetworkInterface>, String> {
    execute_network_status(&context()).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![system_status, storage_status, process_status, network_status])
        .run(tauri::generate_context!())
        .expect("error while running Linux Powerhouse");
}
